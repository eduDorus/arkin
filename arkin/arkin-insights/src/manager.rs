use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use arkin_common::prelude::*;
use rayon::prelude::*;
use rust_decimal::Decimal;
use time::OffsetDateTime;

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

    pub fn insert(&self, event: Feature) {
        self.state.insert(event);
    }

    pub fn insert_batch(&self, events: Vec<Feature>) {
        events.into_iter().for_each(|event| self.insert(event));
    }

    pub fn calculate(&self, timestamp: &OffsetDateTime) -> FeatureSnapshot {
        let instruments = self.state.instruments();
        let features = instruments
            .par_iter()
            .map(|instrument| self.pipeline.calculate(self.state.clone(), timestamp, instrument))
            .flat_map(|f| f)
            .collect::<Vec<_>>();

        FeatureSnapshot {
            event_time: timestamp.to_owned(),
            metrics: features,
        }
    }
}
