use std::collections::HashMap;

use super::{DataType, Feature, FeatureId};
use crate::config::SpreadFeatureConfig;
use anyhow::{anyhow, Result};
use tracing::debug;

#[derive(Debug)]
pub struct SpreadFeature {
    id: FeatureId,
    front_component: FeatureId,
    back_component: FeatureId,
}

impl SpreadFeature {
    pub fn new(id: FeatureId, front_component: FeatureId, back_component: FeatureId) -> Self {
        SpreadFeature {
            id,
            front_component,
            back_component,
        }
    }

    pub fn from_config(config: &SpreadFeatureConfig) -> Self {
        SpreadFeature {
            id: config.id.to_owned(),
            front_component: config.front_component.to_owned(),
            back_component: config.back_component.to_owned(),
        }
    }
}

impl Feature for SpreadFeature {
    fn id(&self) -> &FeatureId {
        &self.id
    }

    fn sources(&self) -> Vec<FeatureId> {
        vec![self.front_component.clone(), self.back_component.clone()]
    }

    fn data_type(&self) -> DataType {
        DataType::Latest
    }

    fn calculate(&self, data: HashMap<FeatureId, Vec<f64>>) -> Result<HashMap<FeatureId, f64>> {
        debug!("Calculating Spread with id: {}", self.id);
        let front = data.get(&self.front_component).ok_or(anyhow!("Missing front_component"))?;
        let back = data.get(&self.back_component).ok_or(anyhow!("Missing back_component"))?;

        let front_value = front.last().ok_or(anyhow!("Missing front_component value"))?;
        let back_value = back.last().ok_or(anyhow!("Missing back_component value"))?;

        let spread = front_value - back_value;

        let mut res = HashMap::new();
        res.insert(self.id.clone(), spread);
        Ok(res)
    }
}
