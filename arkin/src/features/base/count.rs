use crate::{
    config::CountFeatureConfig,
    features::{Feature, FeatureDataRequest, FeatureDataResponse, FeatureId, NodeId, Window},
};
use anyhow::Result;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct CountFeature {
    id: NodeId,
    sources: Vec<NodeId>,
    data: Vec<FeatureDataRequest>,
    input: Window,
    output: FeatureId,
}

impl CountFeature {
    pub fn from_config(config: &CountFeatureConfig) -> Self {
        let sources = vec![config.input.from.clone()];
        let data = vec![config.input.to_owned().into()];
        CountFeature {
            id: config.id.to_owned(),
            sources,
            data,
            input: config.input.to_owned(),
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
        &self.data
    }

    fn calculate(&self, data: FeatureDataResponse) -> Result<HashMap<FeatureId, f64>> {
        debug!("Calculating count with id: {}", self.id);
        let count = data.count(&self.input.feature_id).unwrap_or(0.);
        let mut res = HashMap::new();
        res.insert(self.output.clone(), count);
        Ok(res)
    }
}
