use std::collections::HashMap;

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::prelude::*;
use tracing::debug;

use crate::{
    config::CumSumFeatureConfig,
    service::FeatureModule,
    state::{DataRequest, DataResponse},
};

#[derive(Debug)]
pub struct CumSumFeature {
    id: FeatureId,
    sources: Vec<FeatureId>,
    inputs: Vec<DataRequest>,
    output: FeatureId,
}

impl CumSumFeature {
    pub fn from_config(config: &CumSumFeatureConfig) -> Self {
        CumSumFeature {
            id: config.id.to_owned(),
            sources: vec![config.input.from.clone()],
            inputs: vec![config.input.to_owned().into()],
            output: config.output.to_owned(),
        }
    }
}

impl FeatureModule for CumSumFeature {
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
        debug!("Calculating cumulative sum with id: {}", self.id);

        // Retrieve the values for the feature over the window period
        let values = data.get(self.inputs[0].feature_id());

        // No values to calculate the cumulative sum
        if values.is_empty() {
            return Ok(HashMap::new());
        }

        // Calculate the cumulative sum
        let sum = values.iter().sum::<Decimal>();

        let mut res = HashMap::new();
        res.insert(self.output.clone(), sum);
        Ok(res)
    }
}
