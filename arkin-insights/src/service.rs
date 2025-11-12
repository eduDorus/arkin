use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::prelude::*;
use tracing::{debug, info, warn};

use arkin_core::prelude::*;

use crate::config::PipelineConfig;
use crate::FeaturePipeline;

pub struct InsightService {
    pipeline: FeaturePipeline,
}

impl InsightService {
    pub async fn new(persistence: Arc<dyn PersistenceReader>, config: &PipelineConfig) -> Arc<Self> {
        info!(target: "insights", "Initializing InsightService pipeline");
        let pipeline = FeaturePipeline::new(&persistence, config).await;

        // Log synthetic instrument count from pipeline
        let synthetic_count = pipeline.synthetic_instruments().len();
        info!(
            target: "insights",
            "Pipeline initialized with {} synthetic instruments",
            synthetic_count
        );

        let service = Self { pipeline };
        Arc::new(service)
    }

    pub fn synthetic_instruments(&self) -> Vec<Arc<Instrument>> {
        self.pipeline.synthetic_instruments()
    }

    pub fn insert(&self, insight: Arc<Insight>) {
        debug!(target: "insights", "insert to state");
        self.pipeline.insert(insight);
    }

    pub fn insert_batch(&self, insights: Vec<Arc<Insight>>) {
        debug!(target: "insights", "insert to state {} insights", insights.len());
        self.pipeline.insert_batch(insights);
    }

    pub async fn process_tick(&self, ctx: Arc<CoreCtx>, tick: &InsightsTick) {
        // Calculate features - during warmup this builds up derived features but returns empty vec
        let insights = self.pipeline.calculate(tick.event_time).await;

        // Only publish if we have insights (warmup complete)
        if !insights.is_empty() {
            info!(target: "insights", "calculated {} insights", insights.len());

            let insights_update = InsightsUpdate::builder()
                .event_time(tick.event_time)
                // TODO: FIX INSTRUMENTS
                .instruments(vec![])
                .insights(insights)
                .build();

            ctx.publish(Event::InsightsUpdate(insights_update.into())).await;
            debug!(target: "insights", "published insights update");
        } else if !self.pipeline.is_ready() {
            debug!(
                target: "insights",
                "warmup in progress, {} steps remaining",
                self.pipeline.warmup_remaining()
            );
        }
    }
}

#[async_trait]
impl Runnable for InsightService {
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
                let trade_price_feature = ctx.persistence.get_feature_id("trade_price").await;
                let trade_quantity_feature = ctx.persistence.get_feature_id("trade_quantity").await;
                let trade_notional_feature = ctx.persistence.get_feature_id("trade_notional").await;
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
                self.insert_batch(insights);
            }
            Event::TickUpdate(tick) => {
                debug!(target: "insights", "received tick update" );
                // pub fn to_insights(&self) -> Vec<Arc<Insight>> {
                let tick_bid_price_feature = ctx.persistence.get_feature_id("tick_bid_price").await;
                let tick_bid_quantity_feature = ctx.persistence.get_feature_id("tick_bid_quantity").await;
                let tick_ask_price_feature = ctx.persistence.get_feature_id("tick_ask_price").await;
                let tick_ask_quantity_feature = ctx.persistence.get_feature_id("tick_ask_quantity").await;
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
                self.insert_batch(insights);
            }
            Event::MetricUpdate(metric) => {
                debug!(target: "insights", "received metric update" );
                let metric_feature = ctx.persistence.get_feature_id(&metric.metric_type.to_string()).await;
                let insight = Insight::builder()
                    .event_time(metric.event_time)
                    .instrument(metric.instrument.clone())
                    .feature_id(metric_feature)
                    .value(metric.value.to_f64().unwrap_or(f64::NAN))
                    .insight_type(InsightType::Raw)
                    .persist(false)
                    .build()
                    .into();
                self.insert(insight);
            }
            e => {
                warn!(target: "insights", "received unused event: {}", e.event_type());
            }
        }
    }
}
