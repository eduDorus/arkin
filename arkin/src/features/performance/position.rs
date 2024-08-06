use crate::config::PositionConfig;
use crate::constants::{FILL_PRICE_ID, FILL_QUANTITY_ID, POSITIONS_ID};
use crate::features::{Feature, FeatureId, QueryType};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct PositionFeature {
    id: FeatureId,
    source: Vec<FeatureId>,
    data_type: QueryType,
}

impl PositionFeature {
    pub fn from_config(_config: &PositionConfig) -> Self {
        PositionFeature {
            id: POSITIONS_ID.to_owned(),
            source: vec![FILL_PRICE_ID.to_owned(), FILL_QUANTITY_ID.to_owned()],
            data_type: QueryType::Latest,
        }
    }
}

impl Feature for PositionFeature {
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
        debug!("Calculating Spread with id: {}", self.id);
        let front = data.get(&self.source[0]).ok_or(anyhow!("Missing front_component"))?;
        let back = data.get(&self.source[1]).ok_or(anyhow!("Missing back_component"))?;

        let front_value = front.last().ok_or(anyhow!("Missing front_component value"))?;
        let back_value = back.last().ok_or(anyhow!("Missing back_component value"))?;

        let spread = front_value - back_value;

        let mut res = HashMap::new();
        res.insert(self.id.clone(), spread);
        Ok(res)
    }
}
