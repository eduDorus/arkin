use crate::{
    config::CountFeatureConfig,
    features::{Feature, FeatureDataRequest, FeatureDataResponse, FeatureId, NodeId},
};
use anyhow::Result;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct CountFeature {
    id: NodeId,
    sources: Vec<NodeId>,
    inputs: Vec<FeatureDataRequest>,
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

impl Feature for CountFeature {
    fn id(&self) -> &NodeId {
        &self.id
    }

    fn sources(&self) -> &[NodeId] {
        &self.sources
    }

    fn data(&self) -> &[FeatureDataRequest] {
        &self.inputs
    }

    fn calculate(&self, data: FeatureDataResponse) -> Result<HashMap<FeatureId, f64>> {
        debug!("Calculating count with id: {}", self.id);
        let count = data.count(&self.inputs[0].feature_id()).unwrap_or(0.);
        let mut res = HashMap::new();
        res.insert(self.output.clone(), count);
        Ok(res)
    }
}
