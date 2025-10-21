use std::sync::atomic::AtomicU16;
use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::prelude::*;
use tracing::{debug, info, warn};

use arkin_core::prelude::*;

use crate::config::PipelineConfig;
use crate::pipeline::PipelineGraph;
use crate::prelude::FeatureFactory;
use crate::state::InsightsState;

pub struct Insights {
    persistence: Arc<dyn PersistenceReader>,
    warmup_steps: AtomicU16,
    pipeline: Arc<Pipeline>,
    graph: PipelineGraph,
    state: Arc<InsightsState>,
}

impl Insights {
    pub async fn new(
        persistence: Arc<dyn PersistenceReader>,
        pipeline: Arc<Pipeline>,
        pipeline_config: &PipelineConfig,
        warmup: u16,
    ) -> Arc<Self> {
        let state = Arc::new(InsightsState::builder().build());
        let features = FeatureFactory::from_config(&persistence, &pipeline_config.features).await;
        let graph = PipelineGraph::new(features);
        let service = Self {
            persistence,
            warmup_steps: AtomicU16::new(warmup),
            pipeline,
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
        // TODO FIX INSTRUMENTS
        let insights = self.graph.calculate(&self.state, &self.pipeline, tick.event_time, &[]);
        info!(target: "insights", "calculated {} insights", insights.len());

        if self.warmup_steps.load(std::sync::atomic::Ordering::Relaxed) > 0 {
            let number = self.warmup_steps.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            info!(target: "insights", "warmup tick {}, need {} more", tick.event_time, number);
        } else {
            let insights_tick = InsightsUpdate::builder()
                .event_time(tick.event_time)
                // TODO: FIX INSTRUMENTS
                .instruments(vec![])
                .insights(insights.to_owned())
                .build();

            ctx.publish(Event::WarmupInsightsUpdate(insights_tick.into())).await;
            debug!(target: "insights", "warmup done...");
        }
    }

    pub async fn process_tick(&self, ctx: Arc<CoreCtx>, tick: &InsightsTick) {
        self.state.commit(tick.event_time).await;

        // TODO: We might want to span this calculation with spawn blocking
        // TODO: FIX INSTRUMETNS
        let insights =
            self.graph
                .calculate(&self.state, &self.pipeline, tick.event_time, &[] /* all instruments */);
        debug!(target: "insights", "calculated {} insights", insights.len());

        if self.warmup_steps.load(std::sync::atomic::Ordering::Relaxed) > 0 {
            let number = self.warmup_steps.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
            info!(target: "insights", "warmup tick {} not published",number);
        } else {
            let insights_tick = InsightsUpdate::builder()
                .event_time(tick.event_time)
                // TODO: FIX INSTRUMENTS
                .instruments(vec![])
                .insights(insights.to_owned())
                .build();

            ctx.publish(Event::InsightsUpdate(insights_tick.into())).await;
            debug!(target: "insights", "published inside update");
        }
    }
}

#[async_trait]
impl Runnable for Insights {
    // async fn setup(&self, _service_ctx: Arc<ServiceCtx>, core_ctx: Arc<CoreCtx>) {
    //     let end = core_ctx.now().await.replace_second(0).unwrap().replace_nanosecond(0).unwrap();
    //     let start = end - Duration::from_secs(86400);
    //     let trades = match core_ctx.persistence.list_trades(&self.instruments, start, end).await {
    //         Ok(t) => t,
    //         Err(e) => {
    //             error!(target: "insights", "Failed to list trades: {}", e);
    //             return;
    //         }
    //     };

    //     // Clone the inner Trade from the Arc so to_insights can take ownership without moving out of the Arc
    //     let hist_data = trades.iter().flat_map(|t| t.as_ref().clone().to_insights()).collect::<Vec<_>>();
    //     self.insert_batch(&hist_data).await;

    //     // Call process for every minute from end to start
    //     let frequency = Duration::from_secs(60);
    //     let mut minute = start;
    //     while minute <= core_ctx.now().await {
    //         info!(target: "insights", "processing warmup tick at {}", minute);
    //         let tick = InsightsTick::builder().event_time(minute).frequency(frequency).build();
    //         self.warmup_tick(core_ctx.clone(), &tick).await;
    //         minute += frequency;
    //     }
    //     info!(target: "insights", "finished setup at {}", core_ctx.now().await);
    // }

    async fn handle_event(&self, ctx: Arc<CoreCtx>, event: Event) {
        match event {
            Event::InsightsTick(tick) => {
                debug!(target: "insights", "received insights tick" );
                self.process_tick(ctx, &tick).await;
            }
            Event::AggTradeUpdate(trade) => {
                debug!(target: "insights", "received trade update" );
                let trade_price_feature = self.persistence.get_feature_id("trade_price").await;
                let trade_quantity_feature = self.persistence.get_feature_id("trade_quantity").await;
                let trade_notional_feature = self.persistence.get_feature_id("trade_notional").await;
                let insights = vec![
                    Insight::builder()
                        .event_time(trade.event_time)
                        .instrument(trade.instrument.clone())
                        .feature_id(trade_price_feature)
                        .value(trade.price.to_f64().unwrap_or(f64::NAN))
                        .insight_type(InsightType::Raw)
                        .build()
                        .into(),
                    Insight::builder()
                        .event_time(trade.event_time)
                        .instrument(trade.instrument.clone())
                        .feature_id(trade_quantity_feature)
                        .value(trade.quantity.to_f64().unwrap_or(f64::NAN) * f64::from(trade.side))
                        .insight_type(InsightType::Raw)
                        .build()
                        .into(),
                    Insight::builder()
                        .event_time(trade.event_time)
                        .instrument(trade.instrument.clone())
                        .feature_id(trade_notional_feature)
                        .value((trade.price * trade.quantity).to_f64().unwrap_or(f64::NAN) * f64::from(trade.side))
                        .insight_type(InsightType::Raw)
                        .build()
                        .into(),
                ];
                self.insert_batch(&insights).await;
            }
            Event::TickUpdate(tick) => {
                debug!(target: "insights", "received tick update" );
                // pub fn to_insights(&self) -> Vec<Arc<Insight>> {
                let tick_bid_price_feature = self.persistence.get_feature_id("tick_bid_price").await;
                let tick_bid_quantity_feature = self.persistence.get_feature_id("tick_bid_quantity").await;
                let tick_ask_price_feature = self.persistence.get_feature_id("tick_ask_price").await;
                let tick_ask_quantity_feature = self.persistence.get_feature_id("tick_ask_quantity").await;
                let insights = vec![
                    Insight::builder()
                        .event_time(tick.event_time)
                        .instrument(tick.instrument.clone())
                        .feature_id(tick_bid_price_feature)
                        .value(tick.bid_price.to_f64().unwrap_or(f64::NAN))
                        .insight_type(InsightType::Raw)
                        .build()
                        .into(),
                    Insight::builder()
                        .event_time(tick.event_time)
                        .instrument(tick.instrument.clone())
                        .feature_id(tick_bid_quantity_feature)
                        .value(tick.bid_quantity.to_f64().unwrap_or(f64::NAN))
                        .insight_type(InsightType::Raw)
                        .build()
                        .into(),
                    Insight::builder()
                        .event_time(tick.event_time)
                        .instrument(tick.instrument.clone())
                        .feature_id(tick_ask_price_feature)
                        .value(tick.ask_price.to_f64().unwrap_or(f64::NAN))
                        .insight_type(InsightType::Raw)
                        .build()
                        .into(),
                    Insight::builder()
                        .event_time(tick.event_time)
                        .instrument(tick.instrument.clone())
                        .feature_id(tick_ask_quantity_feature)
                        .value(tick.ask_quantity.to_f64().unwrap_or(f64::NAN))
                        .insight_type(InsightType::Raw)
                        .build()
                        .into(),
                ];
                self.insert_batch(&insights).await;
            }
            Event::MetricUpdate(metric) => {
                debug!(target: "insights", "received metric update" );
                let metric_feature = self.persistence.get_feature_id(&metric.metric_type.to_string()).await;
                let insight = Insight::builder()
                    .event_time(metric.event_time)
                    .instrument(metric.instrument.clone())
                    .feature_id(metric_feature)
                    .value(metric.value.to_f64().unwrap_or(f64::NAN))
                    .insight_type(InsightType::Raw)
                    .persist(false)
                    .build()
                    .into();
                self.insert(insight).await;
            }
            e => {
                warn!(target: "insights", "received unused event: {}", e.event_type());
            }
        }
    }
}

// // Features
// pub static TRADE_PRICE_FEATURE_ID: LazyLock<FeatureId> = LazyLock::new(|| Arc::new("trade_price".to_string()));
// pub static TRADE_QUANTITY_FEATURE_ID: LazyLock<FeatureId> = LazyLock::new(|| Arc::new("trade_quantity".to_string()));
// pub static TRADE_NOTIONAL_FEATURE_ID: LazyLock<FeatureId> = LazyLock::new(|| Arc::new("trade_notional".to_string()));
// pub static TICK_BID_PRICE_FEATURE_ID: LazyLock<FeatureId> = LazyLock::new(|| Arc::new("tick_bid_price".to_string()));
// pub static TICK_BID_QUANTITY_FEATURE_ID: LazyLock<FeatureId> =
//     LazyLock::new(|| Arc::new("tick_bid_quantity".to_string()));
// pub static TICK_ASK_PRICE_FEATURE_ID: LazyLock<FeatureId> = LazyLock::new(|| Arc::new("tick_ask_price".to_string()));
// pub static TICK_ASK_QUANTITY_FEATURE_ID: LazyLock<FeatureId> =
//     LazyLock::new(|| Arc::new("tick_ask_quantity".to_string()));

// pub static RAW_FEATURE_IDS: LazyLock<Vec<FeatureId>> = LazyLock::new(|| {
//     vec![
//         TRADE_PRICE_FEATURE_ID.clone(),
//         TRADE_QUANTITY_FEATURE_ID.clone(),
//         TRADE_NOTIONAL_FEATURE_ID.clone(),
//         TICK_BID_PRICE_FEATURE_ID.clone(),
//         TICK_BID_QUANTITY_FEATURE_ID.clone(),
//         TICK_ASK_PRICE_FEATURE_ID.clone(),
//         TICK_ASK_QUANTITY_FEATURE_ID.clone(),
//     ]
// });
