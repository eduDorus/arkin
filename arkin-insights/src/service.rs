use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use async_trait::async_trait;
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, info, instrument};

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
    #[instrument(skip(self))]
    async fn start(&self, _task_tracker: TaskTracker, _shutdown: CancellationToken) -> Result<(), InsightsError> {
        info!("Starting insights service...");
        info!("Insights service started");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cleanup(&self) -> Result<(), InsightsError> {
        info!("Cleaning up insights service...");
        info!("Insights service cleaned up");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn insert(&self, insight: Insight) -> Result<(), InsightsError> {
        self.state.insert(insight);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn insert_batch(&self, insights: Vec<Insight>) -> Result<(), InsightsError> {
        self.state.insert_batch(insights);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn process(
        &self,
        instruments: &[Arc<Instrument>],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<(), InsightsError> {
        info!("Running insights pipeline from {} to {}", from, to);

        // Generate insights
        let insights = self.pipeline.calculate(self.state.clone(), instruments, to);

        for insight in &insights {
            debug!("Generated insight: {}", insight);
        }

        self.persistence_service.insert_insight_batch_vec(insights).await?;
        Ok(())
    }
}
