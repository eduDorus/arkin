use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use async_trait::async_trait;
use time::OffsetDateTime;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};

use crate::errors::InsightsError;
use crate::factory::FeatureFactory;
use crate::pipeline::PipelineGraph;
use crate::traits::Insights;
use crate::{config::InsightsServiceConfig, state::InsightsState};

#[derive(Debug)]
pub struct InsightsService {
    state: Arc<InsightsState>,
    pubsub: Arc<PubSub>,
    persistence_service: Arc<PersistenceService>,
    pipeline: Arc<Pipeline>,
    graph: PipelineGraph,
    state_lookback: Duration,
}

impl InsightsService {
    pub async fn from_config(
        config: &InsightsServiceConfig,
        pubsub: Arc<PubSub>,
        persistence_service: Arc<PersistenceService>,
    ) -> Self {
        let pipeline = persistence_service
            .pipeline_store
            .read_by_name(&config.pipeline.name)
            .await
            .expect("Could not find pipeline");
        let state = Arc::new(InsightsState::default());
        let features = FeatureFactory::from_config(&config.pipeline.features, pipeline.clone(), state.clone());

        Self {
            state,
            pubsub,
            persistence_service,
            pipeline,
            graph: PipelineGraph::from_config(features),
            state_lookback: Duration::from_secs(config.state_lookback),
        }
    }
}

#[async_trait]
impl Insights for InsightsService {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), InsightsError> {
        info!("Starting insights service...");
        let mut interval_tick = self.pubsub.subscribe::<IntervalTick>();
        let mut trades = self.pubsub.subscribe::<Trade>();
        loop {
            select! {
                Ok(time_tick) = interval_tick.recv() => {
                    debug!("InsightsService received interval tick: {}", time_tick.event_time);
                    if let Err(e) = self.process(time_tick.event_time, &time_tick.instruments, true).await {
                        error!("Error processing interval tick: {}", e);
                    }
                }
                Ok(trade) = trades.recv() => {
                    debug!("InsightsService received trade: {}", trade.event_time);
                    let insights = trade.as_ref().clone().to_insights(self.pipeline.clone());
                    if let Err(e) = self.insert_batch(insights.as_slice()).await {
                        error!("Error inserting trade: {}", e);
                    }
                }
                _ = shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }

    async fn load(
        &self,
        event_time: OffsetDateTime,
        instruments: &[Arc<Instrument>],
        lookback: Duration,
    ) -> Result<(), InsightsError> {
        let start = event_time - lookback;
        debug!("Loading insights from {} to {}", start, event_time);

        // let ticks = self.persistence_service.read_ticks_range(instruments, from, to).await?;
        let trades = self
            .persistence_service
            .trade_store
            .read_range(&instruments, start, event_time)
            .await?;

        let insights = trades
            .into_iter()
            .map(|t| t.as_ref().clone().to_insights(self.pipeline.clone()))
            .flatten()
            .collect::<Vec<_>>();
        debug!("Adding {} insights to state", insights.len());
        self.state.insert_batch(insights.as_slice());
        Ok(())
    }

    async fn insert(&self, insight: Arc<Insight>) -> Result<(), InsightsError> {
        self.state.insert(insight);
        Ok(())
    }

    async fn insert_batch(&self, insights: &[Arc<Insight>]) -> Result<(), InsightsError> {
        self.state.insert_batch(insights);
        Ok(())
    }

    async fn remove(&self, event_time: OffsetDateTime) -> Result<(), InsightsError> {
        let last_time = event_time - self.state_lookback;
        self.state.remove(last_time);
        debug!("Removed insights before {}", last_time);
        Ok(())
    }

    async fn process(
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
            self.pubsub.publish::<InsightTick>(insights_tick);
        }

        if let Err(e) = self.remove(event_time).await {
            error!("Error removing insights: {}", e);
        }
        Ok(insights)
    }
}
