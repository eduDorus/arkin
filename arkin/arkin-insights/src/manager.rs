use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use arkin_core::prelude::*;
use rayon::prelude::*;
use rust_decimal::Decimal;

use crate::ComputationGraph;
use crate::{
    config::InsightsManagerConfig,
    state::{DataRequest, DataResponse, InsightsState},
};

pub trait FeatureModule: Debug + Send + Sync {
    fn id(&self) -> &NodeId;
    fn sources(&self) -> &[NodeId];
    fn data(&self) -> &[DataRequest];
    fn calculate(&self, data: DataResponse) -> Result<HashMap<FeatureId, Decimal>>;
}

pub struct InsightsManager {
    state: Arc<InsightsState>,
    pipeline: ComputationGraph,
}

impl InsightsManager {
    pub fn from_config(config: &InsightsManagerConfig) -> Self {
        Self {
            state: Arc::new(InsightsState::default()),
            pipeline: ComputationGraph::from_config(&config.pipeline),
        }
    }

    pub fn insert(&self, event: Insight) {
        self.state.insert(event);
    }

    pub fn insert_batch(&self, events: Vec<Insight>) {
        events.into_iter().for_each(|event| self.insert(event.into()));
    }

    pub fn process(&self, snapshot: &MarketSnapshot) -> InsightsSnapshot {
        let timestamp = snapshot.timestamp();

        // Update state
        self.insert_batch(snapshot.insights());

        // Get instruments
        let instruments = self.state.instruments();

        // Generate insights
        let insights = instruments
            .par_iter()
            .map(|instrument| self.pipeline.calculate(self.state.clone(), &timestamp, instrument))
            .flat_map(|f| f)
            .collect::<Vec<_>>();

        InsightsSnapshot::new(timestamp, insights)
    }
}
