use std::sync::Arc;

use crate::{
    config::FeatureManagerConfig,
    models::{FeatureSnapshot, Instrument},
    state::StateManager,
};
use rayon::prelude::*;
use time::OffsetDateTime;

use super::Pipeline;

pub struct FeatureManager {
    state: Arc<StateManager>,
    pipeline: Pipeline,
}

impl FeatureManager {
    pub fn from_config(state: Arc<StateManager>, config: &FeatureManagerConfig) -> Self {
        Self {
            state,
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
