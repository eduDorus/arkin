use super::{Feature, FeatureId, QueryType};
use crate::config::VWAPFeatureConfig;
use anyhow::{anyhow, Result};
use rust_decimal::prelude::*;
use std::{collections::HashMap, time::Duration};
use tracing::debug;

#[derive(Debug)]
pub struct VWAPFeature {
    id: FeatureId,
    source: Vec<FeatureId>,
    data_type: QueryType,
}

impl VWAPFeature {
    pub fn from_config(config: &VWAPFeatureConfig) -> Self {
        VWAPFeature {
            id: config.id.to_owned(),
            source: vec!["trade_price".into(), "trade_quantity".into()],
            data_type: QueryType::Window(Duration::from_secs(config.window)),
        }
    }
}

impl Feature for VWAPFeature {
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
        debug!("Calculating VWAP with id: {}", self.id);
        // Check if both trade_price and trade_quantity are present
        let price = data.get(&self.source[0]).ok_or(anyhow!("Missing trade_price"))?;
        let quantity = data.get(&self.source[1]).ok_or(anyhow!("Missing trade_quantity"))?;
        assert_eq!(price.len(), quantity.len());

        let mut total_quantity = f64::zero();
        let mut total_notional = f64::zero();

        price.iter().zip(quantity).for_each(|(p, q)| {
            total_quantity += q;
            total_notional += p * q.abs();
        });

        let vwap = if total_quantity.is_zero() {
            f64::NAN
        } else {
            total_notional / total_quantity
        };

        let mut res = HashMap::new();
        res.insert(self.id.clone(), vwap);
        Ok(res)
    }
}
