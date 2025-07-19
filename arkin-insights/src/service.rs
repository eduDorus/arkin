use std::sync::atomic::AtomicU16;
use std::sync::Arc;

use anyhow::Result;

use async_trait::async_trait;
use tracing::{debug, info, instrument};

use arkin_core::prelude::*;

use crate::config::PipelineConfig;
use crate::errors::InsightsError;
use crate::pipeline::PipelineGraph;
use crate::state::InsightsState;

pub struct Insights {
    identifier: String,
    publisher: Arc<dyn Publisher>,
    warmup_steps: AtomicU16,
    graph: PipelineGraph,
    state: Arc<InsightsState>,
}

impl Insights {
    pub async fn new(
        publisher: Arc<dyn Publisher>,
        pipeline: Arc<Pipeline>,
        pipeline_config: &PipelineConfig,
    ) -> Arc<Self> {
        let state = Arc::new(InsightsState::builder().build());
        let graph = PipelineGraph::from_config(pipeline, state.clone(), pipeline_config);
        let service = Self {
            identifier: "insights".to_owned(),
            publisher,
            warmup_steps: AtomicU16::new(180),
            graph,
            state,
        };
        Arc::new(service)
    }

    pub async fn insert(&self, insight: Arc<Insight>) -> Result<(), InsightsError> {
        self.state.insert(insight);
        Ok(())
    }

    pub async fn insert_batch(&self, insights: &[Arc<Insight>]) {
        self.state.insert_batch(insights);
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn process_tick(&self, tick: &InsightsTick) {
        // TODO: We might want to span this calculation with spawn blocking
        let insights = self.graph.calculate(tick);
        debug!(target: "insights", "calculated {} insights", insights.len());
        let insights_tick = InsightsUpdate::builder()
            .event_time(tick.event_time)
            .instruments(tick.instruments.to_owned())
            .insights(insights.to_owned())
            .build();

        if self.warmup_steps.load(std::sync::atomic::Ordering::Relaxed) > 0 {
            let number = self.warmup_steps.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            info!(target: "insights", "warmup tick {} not published",number);
        } else {
            self.publisher.publish(Event::InsightsUpdate(insights_tick.into())).await;
            debug!(target: "insights", "published inside update");
        }
    }
}

#[async_trait]
impl Runnable for Insights {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            Event::InsightsTick(tick) => {
                self.process_tick(tick).await;
            }
            Event::TradeUpdate(trade) => {
                let insights = trade.as_ref().clone().to_insights();
                self.insert_batch(&insights).await;
            }
            Event::TickUpdate(tick) => {
                let insights = tick.as_ref().clone().to_insights();
                self.insert_batch(&insights).await;
            }
            _ => {}
        }
    }
}
