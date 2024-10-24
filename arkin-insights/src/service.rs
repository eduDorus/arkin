use std::fmt::Debug;
use std::sync::Arc;

use anyhow::Result;

use arkin_core::prelude::*;
use arkin_persistance::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, info};

use crate::pipeline::ComputationGraph;
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

pub struct InsightsService {
    state: Arc<InsightsState>,
    persistance_service: Arc<PersistanceService>,
    pipeline: ComputationGraph,
}

impl InsightsService {
    pub fn from_config(config: &InsightsServiceConfig, persistance_service: Arc<PersistanceService>) -> Self {
        Self {
            state: Arc::new(InsightsState::default()),
            persistance_service,
            pipeline: ComputationGraph::from_config(&config.pipeline),
        }
    }

    pub fn insert(&self, insight: Insight) {
        self.state.insert(insight);
    }

    pub fn insert_batch(&self, insights: Vec<Insight>) {
        self.state.insert_batch(insights);
    }

    pub async fn process(
        &self,
        instruments: &[Arc<Instrument>],
        from: OffsetDateTime,
        to: OffsetDateTime,
    ) -> Result<()> {
        info!("Running insights pipeline from {} to {}", from, to);

        // Generate insights
        let insights = self.pipeline.calculate(self.state.clone(), instruments, to);

        for insight in &insights {
            debug!("Generated insight: {}", insight);
        }

        self.persistance_service.insert_insight_batch(insights).await?;
        Ok(())
    }
}
