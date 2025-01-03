use std::collections::HashMap;

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::Decimal;
use tracing::debug;

use crate::{
    config::SpreadFeatureConfig,
    service::FeatureModule,
    state::{DataRequest, DataResponse},
};

#[derive(Debug)]
pub struct SpreadFeature {
    id: FeatureId,
    sources: Vec<FeatureId>,
    inputs: Vec<DataRequest>,
    output: FeatureId,
    absolute: bool,
}

impl SpreadFeature {
    pub fn from_config(config: &SpreadFeatureConfig) -> Self {
        SpreadFeature {
            id: config.id.to_owned(),
            sources: vec![config.input_front.from.clone(), config.input_back.from.clone()],
            inputs: vec![config.input_front.to_owned().into(), config.input_back.to_owned().into()],
            output: config.output.to_owned(),
            absolute: config.absolute,
        }
    }
}

impl FeatureModule for SpreadFeature {
    fn id(&self) -> &FeatureId {
        &self.id
    }

    fn sources(&self) -> &[FeatureId] {
        &self.sources
    }

    fn data(&self) -> &[DataRequest] {
        &self.inputs
    }

    fn calculate(&self, data: DataResponse) -> Result<HashMap<FeatureId, Decimal>> {
        debug!("Calculating spread with id: {}", self.id);
        let front = data.last(self.inputs[0].feature_id()).unwrap_or_default();
        let back = data.last(self.inputs[1].feature_id()).unwrap_or_default();

        let mut spread = front - back;

        if self.absolute {
            spread = spread.abs();
        }

        let mut res = HashMap::new();
        res.insert(self.output.clone(), spread);
        Ok(res)
    }
}
