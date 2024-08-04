use super::{DataType, Feature, FeatureID};
use crate::config::VolumeFeatureConfig;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::{collections::HashMap, time::Duration};
use tracing::debug;

#[derive(Debug)]
pub struct VolumeFeature {
    id: FeatureID,
    trade_quantity: FeatureID,
    window: Duration,
}

impl VolumeFeature {
    pub fn new(id: FeatureID, window: Duration) -> Self {
        VolumeFeature {
            id,
            trade_quantity: "trade_quantity".into(),
            window,
        }
    }

    pub fn from_config(config: &VolumeFeatureConfig) -> Self {
        VolumeFeature {
            id: config.id.to_owned(),
            trade_quantity: "trade_quantity".into(),
            window: Duration::from_secs(config.window),
        }
    }
}

#[async_trait]
impl Feature for VolumeFeature {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<FeatureID> {
        vec![self.trade_quantity.clone()]
    }

    fn data_type(&self) -> DataType {
        DataType::Window(self.window)
    }

    fn calculate(&self, data: HashMap<FeatureID, Vec<f64>>) -> Result<HashMap<FeatureID, f64>> {
        debug!("Calculating Volume with id: {}", self.id);
        let quantity = data.get(&self.trade_quantity).ok_or(anyhow!("Missing trade_quantity"))?;

        let sum = quantity.iter().sum::<f64>();
        let mut res = HashMap::new();
        res.insert(self.id.clone(), sum);
        Ok(res)
    }
}
