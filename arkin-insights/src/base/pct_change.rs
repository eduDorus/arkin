use std::collections::HashMap;

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::prelude::*;
use tracing::debug;

use crate::{
    config::PctChangeFeatureConfig,
    service::FeatureModule,
    state::{DataRequest, DataResponse},
};

#[derive(Debug)]
pub struct PctChangeFeature {
    id: NodeId,
    sources: Vec<NodeId>,
    inputs: Vec<DataRequest>,
    output: FeatureId,
}

impl PctChangeFeature {
    pub fn from_config(config: &PctChangeFeatureConfig) -> Self {
        PctChangeFeature {
            id: config.id.to_owned(),
            sources: vec![config.input.from.clone()],
            inputs: vec![config.input.to_owned().into()],
            output: config.output.to_owned(),
        }
    }
}

impl FeatureModule for PctChangeFeature {
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
        debug!("Calculating percentage change with id: {}", self.id);

        // Retrieve the values for the feature over the window period
        let values = data.get(self.inputs[0].feature_id());
        let first_value = values.first().unwrap_or(&Decimal::ZERO);
        let last_value = values.last().unwrap_or(&Decimal::ZERO);

        // Calculate the percentage change
        let change = last_value - first_value;
        let percentage_change = change / first_value;

        let mut res = HashMap::new();
        res.insert(self.output.clone(), percentage_change);
        Ok(res)
    }
}
