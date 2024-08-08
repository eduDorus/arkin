use crate::{
    config::MeanFeatureConfig,
    features::{Feature, FeatureDataRequest, FeatureDataResponse, FeatureId, NodeId, Window},
};
use anyhow::Result;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct MeanFeature {
    id: NodeId,
    sources: Vec<NodeId>,
    data: Vec<FeatureDataRequest>,
    input: Window,
    output: FeatureId,
}

impl MeanFeature {
    pub fn from_config(config: &MeanFeatureConfig) -> Self {
        let sources = vec![config.input.from.clone()];
        let data = vec![config.input.to_owned().into()];
        MeanFeature {
            id: config.id.to_owned(),
            sources,
            data,
            input: config.input.to_owned(),
            output: config.output.to_owned(),
        }
    }
}

impl Feature for MeanFeature {
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
        debug!("Calculating mean with id: {}", self.id);
        let mean = data.mean(&self.input.feature_id).unwrap_or(0.);
        let mut res = HashMap::new();
        res.insert(self.output.clone(), mean);
        Ok(res)
    }
}
