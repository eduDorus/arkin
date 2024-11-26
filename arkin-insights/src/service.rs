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
use tracing::{info, instrument};

use crate::errors::InsightsError;
use crate::pipeline::ComputationGraph;
use crate::traits::Insights;
use crate::{config::InsightsServiceConfig, state::InsightsState};

#[derive(Debug)]
pub struct InsightsService {
    state: Arc<InsightsState>,
    pubsub: Arc<PubSub>,
    persistence_service: Arc<PersistenceService>,
    pipeline: ComputationGraph,
}

impl InsightsService {
    pub fn from_config(
        config: &InsightsServiceConfig,
        pubsub: Arc<PubSub>,
        persistence_service: Arc<PersistenceService>,
    ) -> Self {
        Self {
            state: Arc::new(InsightsState::default()),
            pubsub,
            persistence_service,
            pipeline: ComputationGraph::from_config(&config.pipeline),
        }
    }
}

#[async_trait]
impl Insights for InsightsService {
    #[instrument(skip_all)]
    async fn start(&self, _shutdown: CancellationToken) -> Result<(), InsightsError> {
        info!("Starting insights service...");
        let mut interval_tick = self.pubsub.subscribe::<IntervalTick>();
        loop {
            select! {
                Ok(time_tick) = interval_tick.recv() => {
                    info!("InsightsService received interval tick: {}", time_tick.event_time);
                    self.load(time_tick.event_time, &time_tick.instruments, time_tick.frequency).await?;
                    self.process(time_tick.event_time, &time_tick.instruments, true).await?;
                }
                _ = _shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }

    #[instrument(skip_all)]
    async fn load(
        &self,
        event_time: OffsetDateTime,
        instruments: &[Arc<Instrument>],
        lookback: Duration,
    ) -> Result<(), InsightsError> {
        let start = event_time - lookback;
        info!("Loading insights from {} to {}", start, event_time);

        // let ticks = self.persistence_service.read_ticks_range(instruments, from, to).await?;
        let trades = self
            .persistence_service
            .read_trades_range(instruments, start, event_time)
            .await?;

        let insights = trades.into_iter().map(|t| t.to_insights()).flatten().collect::<Vec<_>>();
        info!("Adding {} insights to state", insights.len());
        self.state.insert_batch(insights);
        Ok(())
    }
    #[instrument(skip_all)]
    async fn insert(&self, insight: Insight) -> Result<(), InsightsError> {
        self.state.insert(insight);
        Ok(())
    }

    #[instrument(skip_all)]
    async fn insert_batch(&self, insights: Vec<Insight>) -> Result<(), InsightsError> {
        self.state.insert_batch(insights);
        Ok(())
    }

    #[instrument(skip_all)]
    async fn process(
        &self,
        event_time: OffsetDateTime,
        instruments: &[Arc<Instrument>],
        publish: bool,
    ) -> Result<Vec<Insight>, InsightsError> {
        info!("Running insights pipeline at event time: {}", event_time);
        let insights = self.pipeline.calculate(self.state.clone(), instruments, event_time);
        let insights_tick = InsightTickBuilder::default()
            .event_time(event_time)
            .instruments(instruments.to_vec())
            .insights(insights.clone())
            .build()
            .unwrap();

        if publish {
            info!(
                "Publishing insights tick: {} with {} insights",
                insights_tick.event_time,
                insights.len()
            );
            self.pubsub.publish::<InsightTick>(insights_tick);
        }
        Ok(insights)
    }
}
