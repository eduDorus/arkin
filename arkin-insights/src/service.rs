use std::sync::Arc;

use anyhow::Result;

use async_trait::async_trait;
use time::UtcDateTime;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use arkin_core::prelude::*;

use crate::config::PipelineConfig;
use crate::errors::InsightsError;
use crate::pipeline::PipelineGraph;
use crate::state::InsightsState;

pub struct InsightsService {
    pubsub: PubSubHandle,
    state: Arc<InsightsState>,
    graph: PipelineGraph,
}

impl InsightsService {
    pub async fn init(pubsub: PubSubHandle, pipeline: Arc<Pipeline>, pipeline_config: &PipelineConfig) -> Arc<Self> {
        let state = Arc::new(InsightsState::builder().build());
        let graph = PipelineGraph::from_config(pipeline, state.clone(), pipeline_config);
        let service = Self {
            state,
            pubsub,
            graph,
        };
        Arc::new(service)
    }

    pub async fn insert(&self, insight: Arc<Insight>) -> Result<(), InsightsError> {
        self.state.insert(insight);
        Ok(())
    }

    pub async fn insert_batch(&self, insights: &[Arc<Insight>]) -> Result<(), InsightsError> {
        self.state.insert_batch(insights);
        Ok(())
    }

    pub async fn process(
        &self,
        event_time: UtcDateTime,
        instruments: &[Arc<Instrument>],
        publish: bool,
    ) -> Result<Vec<Arc<Insight>>, InsightsError> {
        debug!("Running insights pipeline at event time: {}", event_time);
        let insights = self.graph.calculate(instruments, event_time);
        let insights_tick = InsightsUpdate::builder()
            .event_time(event_time)
            .instruments(instruments.to_vec())
            .insights(insights.clone())
            .build();
        let insights_tick = Arc::new(insights_tick);

        if publish {
            debug!(
                "Publishing insights tick: {} with {} insights",
                insights_tick.event_time,
                insights.len()
            );
            self.pubsub.publish(insights_tick).await;
        }

        Ok(insights)
    }
}

#[async_trait]
impl RunnableService for InsightsService {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting insights service...");

        let mut warm_up = 1439;

        loop {
            select! {
                Some(event) = self.pubsub.recv() => {
                    match event {
                        Event::InsightsTick(tick) => {
                            info!("InsightsService received interval tick: {}", tick.event_time);
                            let publish = if warm_up > 0 {
                                warm_up -= 1;
                                info!("Warming up insights service for {} more steps, skipping publish", warm_up);
                                false
                            } else {
                                true
                            };
                            if let Err(e) = self.process(tick.event_time, &tick.instruments, publish).await {
                                error!("Error processing interval tick: {}", e);
                            }
                        }
                        Event::TradeUpdate(trade) => {
                            debug!("InsightsService received trade: {}", trade.event_time);
                            let insights = trade.as_ref().clone().to_insights();
                            if let Err(e) = self.insert_batch(&insights).await {
                                error!("Error inserting trade: {}", e);
                            }
                        }
                        Event::TickUpdate(tick) => {
                            debug!("InsightsService received tick: {}", tick.event_time);
                            let insights = tick.as_ref().clone().to_insights();
                            if let Err(e) = self.insert_batch(&insights).await {
                                error!("Error inserting tick: {}", e);
                            }
                        }
                        _ => {}
                    }
                    self.pubsub.ack().await;
                }
                _ = shutdown.cancelled() => {
                    info!("Insights service shutdown...");
                    break;
                }
            }
        }
        info!("Insights service stopped.");
        Ok(())
    }
}
