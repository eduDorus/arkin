use std::collections::HashMap;

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::Decimal;
use tracing::debug;

use crate::{
    config::SumFeatureConfig,
    manager::FeatureModule,
    state::{DataRequest, DataResponse},
};

#[derive(Debug)]
pub struct SumFeature {
    id: NodeId,
    sources: Vec<NodeId>,
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
    fn id(&self) -> &NodeId {
        &self.id
    }

    fn sources(&self) -> &[NodeId] {
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
