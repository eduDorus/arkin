use super::{Feature, FeatureID};
use crate::config::SMAFeatureConfig;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::info;

#[derive(Debug)]
pub struct SMAFeature {
    id: FeatureID,
    source: FeatureID,
    _period: u64,
}

impl SMAFeature {
    pub fn new(id: FeatureID, source: FeatureID, period: u64) -> Self {
        SMAFeature {
            id,
            source,
            _period: period,
        }
    }

    pub fn from_config(config: &SMAFeatureConfig) -> Self {
        SMAFeature {
            id: config.id.to_owned(),
            source: config.source.to_owned(),
            _period: config.period,
        }
    }
}

#[async_trait]
impl Feature for SMAFeature {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<&FeatureID> {
        vec![&self.source]
    }

    fn calculate(&self, data: HashMap<FeatureID, Vec<f64>>) -> Result<HashMap<FeatureID, f64>> {
        info!("Calculating SMA with id: {}", self.id);
        let values = data.get(&self.source).ok_or(anyhow!("Missing {}", self.source))?;

        let sum = values.iter().sum::<f64>();
        let count = values.len() as f64;

        let sma = sum / count;
        let mut res = HashMap::new();
        res.insert(self.id.clone(), sma);
        Ok(res)
    }
}
