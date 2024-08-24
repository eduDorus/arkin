use std::collections::HashMap;

use anyhow::Result;
use arkin_common::prelude::*;
use rust_decimal::Decimal;
use tracing::debug;

use crate::{
    config::MeanFeatureConfig,
    manager::FeatureModule,
    state::{DataRequest, DataResponse},
};

#[derive(Debug)]
pub struct MeanFeature {
    id: NodeId,
    sources: Vec<NodeId>,
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
        let mean = data.mean(self.inputs[0].feature_id()).unwrap_or_default();
        let mut res = HashMap::new();
        res.insert(self.output.clone(), mean);
        Ok(res)
    }
}
