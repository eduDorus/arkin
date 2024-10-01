use std::collections::HashMap;

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::Decimal;
use tracing::debug;

use crate::{
    config::CountFeatureConfig,
    service::FeatureModule,
    state::{DataRequest, DataResponse},
};

#[derive(Debug)]
pub struct CountFeature {
    id: NodeId,
    sources: Vec<NodeId>,
    inputs: Vec<DataRequest>,
    output: FeatureId,
}

impl CountFeature {
    pub fn from_config(config: &CountFeatureConfig) -> Self {
        CountFeature {
            id: config.id.to_owned(),
            sources: vec![config.input.from.to_owned()],
            inputs: vec![config.input.to_owned().into()],
            output: config.output.to_owned(),
        }
    }
}

impl FeatureModule for CountFeature {
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
        debug!("Calculating count with id: {}", self.id);
        let count = data.count(self.inputs[0].feature_id()).unwrap_or_default();
        let mut res = HashMap::new();
        res.insert(self.output.clone(), count);
        Ok(res)
    }
}
