use std::collections::HashMap;

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::Decimal;
use tracing::debug;

use crate::{
    config::SumFeatureConfig,
    service::FeatureModule,
    state::{DataRequest, DataResponse},
};

#[derive(Clone)]
pub struct SumComputation;

impl SingleVecComputation for SumComputation {
    fn compute(&self, data: &[Decimal]) -> Option<f64> {
        if data.is_empty() {
            None
        } else {
            Some(data.iter().sum())
        }
    }
}

#[derive(Debug)]
pub struct SumFeature {
    id: FeatureId,
    sources: Vec<FeatureId>,
    inputs: Vec<DataRequest>,
    output: FeatureId,
}

impl SumFeature {
    pub fn from_config(config: &SumFeatureConfig) -> Self {
        SumFeature {
            id: config.id.to_owned(),
            sources: vec![config.input.from.clone()],
            inputs: vec![config.input.to_owned().into()],
            output: config.output.to_owned(),
        }
    }
}

impl FeatureModule for SumFeature {
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
        debug!("Calculating sum with id: {}", self.id);
        let sum = data.sum(self.inputs[0].feature_id()).unwrap_or_default();
        let mut res = HashMap::new();
        res.insert(self.output.clone(), sum);
        Ok(res)
    }
}
