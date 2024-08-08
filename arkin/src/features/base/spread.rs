use crate::config::SpreadFeatureConfig;
use crate::features::{Feature, FeatureDataRequest, FeatureDataResponse, FeatureId, Latest, NodeId};
use anyhow::Result;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct SpreadFeature {
    id: NodeId,
    sources: Vec<NodeId>,
    data: Vec<FeatureDataRequest>,
    input_front: Latest,
    input_back: Latest,
    output: FeatureId,
    absolute: bool,
}

impl SpreadFeature {
    pub fn from_config(config: &SpreadFeatureConfig) -> Self {
        let mut sources = vec![config.input_front.from.clone(), config.input_back.from.clone()];
        sources.dedup();
        let data = vec![config.input_front.to_owned().into(), config.input_back.to_owned().into()];
        SpreadFeature {
            id: config.id.to_owned(),
            sources,
            data,
            input_front: config.input_front.to_owned(),
            input_back: config.input_back.to_owned(),
            output: config.output.to_owned(),
            absolute: config.absolute,
        }
    }
}

impl Feature for SpreadFeature {
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
        debug!("Calculating spread with id: {}", self.id);
        let front = data.last(&self.input_front.feature_id).unwrap_or(0.);
        let back = data.last(&self.input_back.feature_id).unwrap_or(0.);

        let mut spread = front - back;

        if self.absolute {
            spread = spread.abs();
        }

        let mut res = HashMap::new();
        res.insert(self.output.clone(), spread);
        Ok(res)
    }
}
