use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use arkin_common::prelude::*;
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

    pub fn calculate(&self, snapshot: &Snapshot) -> Vec<Insight> {
        let instruments = self.state.instruments();
        instruments
            .par_iter()
            .map(|instrument| self.pipeline.calculate(self.state.clone(), &snapshot.event_time, instrument))
            .flat_map(|f| f)
            .collect::<Vec<_>>()
    }
}