use super::{DataType, Feature, FeatureID};
use crate::config::SMAFeatureConfig;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct SMAFeature {
    id: FeatureID,
    source: FeatureID,
    period: usize,
}

impl SMAFeature {
    pub fn new(id: FeatureID, source: FeatureID, period: usize) -> Self {
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

#[async_trait]
impl Feature for SMAFeature {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<FeatureID> {
        vec![self.source.clone()]
    }

    fn data_type(&self) -> DataType {
        DataType::Period(self.period)
    }

    fn calculate(&self, data: HashMap<FeatureID, Vec<f64>>) -> Result<HashMap<FeatureID, f64>> {
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
