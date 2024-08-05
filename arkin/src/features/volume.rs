use super::{Feature, FeatureId, QueryType};
use crate::config::VolumeFeatureConfig;
use anyhow::{anyhow, Result};
use std::{collections::HashMap, time::Duration};
use tracing::debug;

#[derive(Debug)]
pub struct VolumeFeature {
    id: FeatureId,
    source: Vec<FeatureId>,
    data_type: QueryType,
}

impl VolumeFeature {
    pub fn from_config(config: &VolumeFeatureConfig) -> Self {
        VolumeFeature {
            id: config.id.to_owned(),
            source: vec!["trade_quantity".into()],
            data_type: QueryType::Window(Duration::from_secs(config.window)),
        }
    }
}

impl Feature for VolumeFeature {
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
        debug!("Calculating Volume with id: {}", self.id);
        let quantity = data.get(&self.source[0]).ok_or(anyhow!("Missing trade_quantity"))?;

        let sum = quantity.iter().sum::<f64>();
        let mut res = HashMap::new();
        res.insert(self.id.clone(), sum);
        Ok(res)
    }
}
