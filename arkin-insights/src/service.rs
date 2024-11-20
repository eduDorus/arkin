use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use async_trait::async_trait;
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{info, instrument};

use crate::errors::InsightsError;
use crate::pipeline::ComputationGraph;
use crate::traits::Insights;
use crate::{config::InsightsServiceConfig, state::InsightsState};

pub trait Computation: Debug + Send + Sync {
    fn inputs(&self) -> Vec<FeatureId>;
    fn outputs(&self) -> Vec<FeatureId>;
    fn calculate(
        &self,
        instruments: &[Arc<Instrument>],
        timestamp: OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>>;
}

#[derive(Debug)]
pub struct InsightsService {
    state: Arc<InsightsState>,
    persistence_service: Arc<PersistenceService>,
    pipeline: ComputationGraph,
}

impl InsightsService {
    pub fn from_config(config: &InsightsServiceConfig, persistence_service: Arc<PersistenceService>) -> Self {
        Self {
            state: Arc::new(InsightsState::default()),
            persistence_service,
            pipeline: ComputationGraph::from_config(&config.pipeline),
        }
    }
}

#[async_trait]
impl Insights for InsightsService {
    #[instrument(skip_all)]
    async fn start(&self, _task_tracker: TaskTracker, _shutdown: CancellationToken) -> Result<(), InsightsError> {
        info!("Starting insights service...");
        info!("Insights service started");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn cleanup(&self) -> Result<(), InsightsError> {
        info!("Cleaning up insights service...");
        info!("Insights service cleaned up");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn load(
        &self,
        instruments: &[Arc<Instrument>],
        event_time: OffsetDateTime,
        frequency: Duration,
    ) -> Result<(), InsightsError> {
        info!("Loading insights from {} to {}", event_time, event_time - frequency);

        // let ticks = self.persistence_service.read_ticks_range(instruments, from, to).await?;
        let trades = self
            .persistence_service
            .read_trades_range(instruments, event_time - frequency, event_time)
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
        instruments: &[Arc<Instrument>],
        event_time: OffsetDateTime,
    ) -> Result<Vec<Insight>, InsightsError> {
        info!("Running insights pipeline at event time: {}", event_time);

        let insights = self.pipeline.calculate(self.state.clone(), instruments, event_time);
        self.persistence_service.insert_insight_batch_vec(insights.clone()).await?;
        Ok(insights)
    }
}
