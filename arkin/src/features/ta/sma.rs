use crate::{
    config::SMAFeatureConfig,
    features::{Feature, FeatureDataRequest, FeatureDataResponse, FeatureId, NodeId},
};
use anyhow::Result;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct SMAFeature {
    id: NodeId,
    sources: Vec<NodeId>,
    inputs: Vec<FeatureDataRequest>,
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

impl Feature for SMAFeature {
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
        debug!("Calculating mean with id: {}", self.id);
        let sum = data.mean(self.inputs[0].feature_id()).unwrap_or(0.);
        let count = data.count(self.inputs[0].feature_id()).unwrap_or(0.);

        let mean = if count == 0. { f64::NAN } else { sum / count };
        let mut res = HashMap::new();
        res.insert(self.output.clone(), mean);
        Ok(res)
    }
}
