use std::collections::HashMap;

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::Decimal;
use tracing::debug;

use crate::{
    config::MeanFeatureConfig,
    service::FeatureModule,
    state::{DataRequest, DataResponse},
};

#[derive(Debug)]
pub struct MeanFeature {
    id: FeatureId,
    sources: Vec<FeatureId>,
    inputs: Vec<DataRequest>,
    output: FeatureId,
}

impl MeanFeature {
    pub fn from_config(config: &MeanFeatureConfig) -> Self {
        MeanFeature {
            id: config.id.to_owned(),
            sources: vec![config.input.from.clone()],
            inputs: vec![config.input.to_owned().into()],
            output: config.output.to_owned(),
        }
    }
}

impl FeatureModule for MeanFeature {
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
        debug!("Calculating mean with id: {}", self.id);

        let values = data.get(self.inputs[0].feature_id());

        // No values to calculate the mean
        if values.is_empty() {
            return Ok(HashMap::new());
        }

        let mean = values.iter().sum::<Decimal>() / Decimal::from(values.len());

        let mut res = HashMap::new();
        res.insert(self.output.clone(), mean);
        Ok(res)
    }
}
