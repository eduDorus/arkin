use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

use async_trait::async_trait;
use time::OffsetDateTime;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use arkin_core::prelude::*;

use crate::config::PipelineConfig;
use crate::errors::InsightsError;
use crate::feature_factory::FeatureFactory;
use crate::pipeline::PipelineGraph;
use crate::state::InsightsState;

#[derive(Debug)]
pub struct InsightsService {
    pubsub: Arc<PubSub>,
    state: Arc<InsightsState>,
    pipeline: Arc<Pipeline>,
    graph: PipelineGraph,
}

impl InsightsService {
    pub async fn init(pubsub: Arc<PubSub>, pipeline: Arc<Pipeline>, pipeline_config: &PipelineConfig) -> Arc<Self> {
        let state = Arc::new(InsightsState::builder().build());
        let features = FeatureFactory::from_config(&pipeline_config.features, pipeline.clone(), state.clone());

        let service = Self {
            state,
            pubsub,
            pipeline,
            graph: PipelineGraph::from_config(features),
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
        event_time: OffsetDateTime,
        instruments: &[Arc<Instrument>],
        publish: bool,
    ) -> Result<Vec<Arc<Insight>>, InsightsError> {
        info!("Running insights pipeline at event time: {}", event_time);
        let insights = self.graph.calculate(instruments, event_time);
        let insights_tick = InsightTick::builder()
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

        let mut rx = self.pubsub.subscribe();

        loop {
            select! {
                Ok(event) = rx.recv() => {
                    match event {
                        Event::IntervalTick(tick) => {
                            debug!("InsightsService received interval tick: {}", tick.event_time);
                            if let Err(e) = self.process(tick.event_time, &tick.instruments, true).await {
                                error!("Error processing interval tick: {}", e);
                            }
                        }
                        Event::Trade(trade) => {
                            debug!("InsightsService received trade: {}", trade.event_time);
                            let insights = trade.as_ref().clone().to_insights(self.pipeline.clone());
                            if let Err(e) = self.insert_batch(&insights).await {
                                error!("Error inserting trade: {}", e);
                            }
                        }
                        _ => {}
                    }
                }
                _ = shutdown.cancelled() => {
                    info!("Insights service shutdown...");
                    break;
                }
            }
        }
        Ok(())
    }
}
