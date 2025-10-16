use std::sync::atomic::AtomicU16;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use tracing::{debug, error, info, warn};

use arkin_core::prelude::*;

use crate::config::PipelineConfig;
use crate::pipeline::PipelineGraph;
use crate::state::InsightsState;

pub struct Insights {
    instruments: Vec<Arc<Instrument>>,
    warmup_steps: AtomicU16,
    graph: PipelineGraph,
    state: Arc<InsightsState>,
}

impl Insights {
    pub fn new(
        pipeline: Arc<Pipeline>,
        pipeline_config: &PipelineConfig,
        instruments: Vec<Arc<Instrument>>,
        warmup: u16,
    ) -> Arc<Self> {
        let state = Arc::new(InsightsState::builder().build());
        let graph = PipelineGraph::from_config(pipeline, state.clone(), pipeline_config);
        let service = Self {
            instruments,
            warmup_steps: AtomicU16::new(warmup),
            graph,
            state,
        };
        Arc::new(service)
    }

    pub async fn insert(&self, insight: Arc<Insight>) {
        debug!(target: "insights", "insert to state");
        self.state.insert_buffered(insight).await;
    }

    pub async fn insert_batch(&self, insights: &[Arc<Insight>]) {
        debug!(target: "insights", "insert to state {} insights", insights.len());
        self.state.insert_batch_buffered(insights).await;
    }

    pub async fn warmup_tick(&self, ctx: Arc<CoreCtx>, tick: &InsightsTick) {
        self.state.commit(tick.event_time).await;
        let insights = self.graph.calculate(tick.event_time, &self.instruments);
        info!(target: "insights", "calculated {} insights", insights.len());

        if self.warmup_steps.load(std::sync::atomic::Ordering::Relaxed) > 0 {
            let number = self.warmup_steps.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            info!(target: "insights", "warmup tick {}, need {} more", tick.event_time, number);
        } else {
            let insights_tick = InsightsUpdate::builder()
                .event_time(tick.event_time)
                .instruments(self.instruments.clone())
                .insights(insights.to_owned())
                .build();

            ctx.publish(Event::WarmupInsightsUpdate(insights_tick.into())).await;
            debug!(target: "insights", "warmup done...");
        }
    }

    pub async fn process_tick(&self, ctx: Arc<CoreCtx>, tick: &InsightsTick) {
        self.state.commit(tick.event_time).await;
        // TODO: We might want to span this calculation with spawn blocking
        let insights = self.graph.calculate(tick.event_time, &self.instruments);
        debug!(target: "insights", "calculated {} insights", insights.len());

        if self.warmup_steps.load(std::sync::atomic::Ordering::Relaxed) > 0 {
            let number = self.warmup_steps.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            info!(target: "insights", "warmup tick {} not published",number);
        } else {
            let insights_tick = InsightsUpdate::builder()
                .event_time(tick.event_time)
                .instruments(self.instruments.clone())
                .insights(insights.to_owned())
                .build();

            ctx.publish(Event::InsightsUpdate(insights_tick.into())).await;
            debug!(target: "insights", "published inside update");
        }
    }
}

#[async_trait]
impl Runnable for Insights {
    async fn setup(&self, _service_ctx: Arc<ServiceCtx>, core_ctx: Arc<CoreCtx>) {
        let end = core_ctx.now().await.replace_second(0).unwrap().replace_nanosecond(0).unwrap();
        let start = end - Duration::from_secs(86400);
        let trades = match core_ctx.persistence.list_trades(&self.instruments, start, end).await {
            Ok(t) => t,
            Err(e) => {
                error!(target: "insights", "Failed to list trades: {}", e);
                return;
            }
        };

        // Clone the inner Trade from the Arc so to_insights can take ownership without moving out of the Arc
        let hist_data = trades.iter().flat_map(|t| t.as_ref().clone().to_insights()).collect::<Vec<_>>();
        self.insert_batch(&hist_data).await;

        // Call process for every minute from end to start
        let frequency = Duration::from_secs(60);
        let mut minute = start;
        while minute <= core_ctx.now().await {
            info!(target: "insights", "processing warmup tick at {}", minute);
            let tick = InsightsTick::builder().event_time(minute).frequency(frequency).build();
            self.warmup_tick(core_ctx.clone(), &tick).await;
            minute += frequency;
        }
        info!(target: "insights", "finished setup at {}", core_ctx.now().await);
    }

    async fn handle_event(&self, ctx: Arc<CoreCtx>, event: Event) {
        match event {
            Event::InsightsTick(tick) => {
                debug!(target: "insights", "received insights tick" );
                self.process_tick(ctx, &tick).await;
            }
            Event::AggTradeUpdate(trade) => {
                debug!(target: "insights", "received trade update" );
                let insights = trade.to_insights();
                self.insert_batch(&insights).await;
            }
            Event::TickUpdate(tick) => {
                debug!(target: "insights", "received tick update" );
                let insights = tick.to_insights();
                self.insert_batch(&insights).await;
            }
            Event::MetricUpdate(metric) => {
                debug!(target: "insights", "received metric update" );
                let insight = metric.to_insight();
                self.insert(insight).await;
            }
            e => {
                warn!(target: "insights", "received unused event: {}", e.event_type());
            }
        }
    }
}
