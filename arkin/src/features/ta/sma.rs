use crate::{
    config::SMAFeatureConfig,
    features::{Feature, FeatureId, QueryType},
};
// Import FeatureId and QueryType from src/features/base/mod.rs
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct SMAFeature {
    id: FeatureId,
    source: Vec<FeatureId>,
    data_type: QueryType,
}

impl SMAFeature {
    pub fn from_config(config: &SMAFeatureConfig) -> Self {
        SMAFeature {
            id: config.id.to_owned(),
            source: vec![config.source.to_owned()],
            data_type: QueryType::Period(config.period),
        }
    }
}

impl Feature for SMAFeature {
    fn id(&self) -> &FeatureId {
        &self.id
    }

    fn sources(&self) -> &[FeatureId] {
        &self.source
    }

    fn data_type(&self) -> &QueryType {
        &self.data_type
    }

    fn calculate(&self, data: HashMap<FeatureId, Vec<f64>>) -> Result<HashMap<FeatureId, f64>> {
        debug!("Calculating SMA with id: {}", self.id);
        let values = data.get(&self.source[0]).ok_or(anyhow!("Missing {}", self.source[0]))?;

        let sum = values.iter().sum::<f64>();
        let count = values.len() as f64;

        let sma = sum / count;
        let mut res = HashMap::new();
        res.insert(self.id.clone(), sma);
        Ok(res)
    }
}
