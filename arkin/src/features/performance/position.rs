use crate::constants::{FILL_PRICE_ID, FILL_QUANTITY_ID, POSITION_PRICE_ID, POSITION_QUANTITY_ID};
use crate::features::{Feature, FeatureId, QueryType};
use anyhow::Result;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct PositionFeature {
    source: Vec<FeatureId>,
    data_type: QueryType,
}

impl PositionFeature {
    pub fn from_config() -> Self {
        PositionFeature {
            source: vec![
                POSITION_PRICE_ID.clone(),
                POSITION_QUANTITY_ID.clone(),
                FILL_PRICE_ID.to_owned(),
                FILL_QUANTITY_ID.to_owned(),
            ],
            data_type: QueryType::Latest,
        }
    }
}

impl Feature for PositionFeature {
    fn id(&self) -> &FeatureId {
        &self.source[0]
    }

    fn sources(&self) -> &[FeatureId] {
        &self.source
    }

    fn data_type(&self) -> &QueryType {
        &self.data_type
    }

    fn calculate(&self, data: HashMap<FeatureId, Vec<f64>>) -> Result<HashMap<FeatureId, f64>> {
        debug!("Calculating Position");
        let (position_price, position_quantity) =
            if !data.get(&self.source[0]).unwrap().is_empty() && !data.get(&self.source[1]).unwrap().is_empty() {
                (
                    *data.get(&self.source[0]).unwrap().clone().last().unwrap(),
                    *data.get(&self.source[1]).unwrap().clone().last().unwrap(),
                )
            } else {
                (0.0, 0.0)
            };

        let fill_price = data.get(&self.source[2]).unwrap();
        let fill_quantity = data.get(&self.source[3]).unwrap();

        let mut position_price_sum = position_price * position_quantity;
        let mut position_quantity_sum = position_quantity;

        for (price, quantity) in fill_price.iter().zip(fill_quantity) {
            position_price_sum += price * quantity;
            position_quantity_sum += quantity;
        }

        let position_price = if position_quantity_sum == 0.0 {
            0.0
        } else {
            position_price_sum / position_quantity_sum
        };

        let mut res = HashMap::new();
        res.insert(self.source[0].clone(), position_price);
        res.insert(self.source[1].clone(), position_quantity_sum);
        Ok(res)
    }
}
