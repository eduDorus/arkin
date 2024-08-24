use std::collections::HashMap;

use anyhow::Result;
use arkin_common::prelude::*;
use rust_decimal::Decimal;
use tracing::debug;

use crate::{
    config::SMAFeatureConfig,
    manager::FeatureModule,
    state::{DataRequest, DataResponse},
};

#[derive(Debug)]
pub struct SMAFeature {
    id: NodeId,
    sources: Vec<NodeId>,
    inputs: Vec<DataRequest>,
    output: FeatureId,
}

impl SMAFeature {
    pub fn from_config(config: &SMAFeatureConfig) -> Self {
        let sources = vec![config.input.from.clone()];
        let data = vec![config.input.to_owned().into()];

        SMAFeature {
            id: config.id.to_owned(),
            sources,
            inputs: data,
            output: config.output.to_owned(),
        }
    }
}

impl FeatureModule for SMAFeature {
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
        debug!("Calculating mean with id: {}", self.id);
        let sum = data.mean(self.inputs[0].feature_id()).unwrap_or_default();
        let count = data.count(self.inputs[0].feature_id()).unwrap_or_default();

        let mean = if count.is_zero() {
            Decimal::ZERO
        } else {
            sum / count
        };
        let mut res = HashMap::new();
        res.insert(self.output.clone(), mean);
        Ok(res)
    }
}
