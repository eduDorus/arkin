use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use arkin_common::prelude::*;
use rayon::prelude::*;
use time::OffsetDateTime;

use crate::Pipeline;
use crate::{
    config::FeatureManagerConfig,
    state::{FeatureDataRequest, FeatureDataResponse, FeatureState},
};

pub trait FeatureModule: Debug + Send + Sync {
    fn id(&self) -> &NodeId;
    fn sources(&self) -> &[NodeId];
    fn data(&self) -> &[FeatureDataRequest];
    fn calculate(&self, data: FeatureDataResponse) -> Result<HashMap<FeatureId, f64>>;
}

pub struct FeatureManager {
    state: Arc<FeatureState>,
    pipeline: Pipeline,
}

impl FeatureManager {
    pub fn from_config(config: &FeatureManagerConfig) -> Self {
        Self {
            state: Arc::new(FeatureState::default()),
            pipeline: Pipeline::from_config(&config.pipeline),
        }
    }

    pub fn calculate(&self, timestamp: &OffsetDateTime, instruments: &[Instrument]) -> FeatureSnapshot {
        let features = instruments
            .par_iter()
            .map(|instrument| self.pipeline.calculate(self.state.clone(), timestamp, instrument))
            .flat_map(|f| f)
            .collect::<Vec<_>>();

        FeatureSnapshot {
            event_time: timestamp.to_owned(),
            features,
        }
    }
}
