use super::{DataType, Feature, FeatureId};
use crate::config::SMAFeatureConfig;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct SMAFeature {
    id: FeatureId,
    source: FeatureId,
    period: usize,
}

impl SMAFeature {
    pub fn new(id: FeatureId, source: FeatureId, period: usize) -> Self {
        SMAFeature { id, source, period }
    }

    pub fn from_config(config: &SMAFeatureConfig) -> Self {
        SMAFeature {
            id: config.id.to_owned(),
            source: config.source.to_owned(),
            period: config.period,
        }
    }
}

impl Feature for SMAFeature {
    fn id(&self) -> &FeatureId {
        &self.id
    }

    fn sources(&self) -> Vec<FeatureId> {
        vec![self.source.clone()]
    }

    fn data_type(&self) -> DataType {
        DataType::Period(self.period)
    }

    fn calculate(&self, data: HashMap<FeatureId, Vec<f64>>) -> Result<HashMap<FeatureId, f64>> {
        debug!("Calculating SMA with id: {}", self.id);
        let values = data.get(&self.source).ok_or(anyhow!("Missing {}", self.source))?;

        let sum = values.iter().sum::<f64>();
        let count = values.len() as f64;

        let sma = sum / count;
        let mut res = HashMap::new();
        res.insert(self.id.clone(), sma);
        Ok(res)
    }
}
