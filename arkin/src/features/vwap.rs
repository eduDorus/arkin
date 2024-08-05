use super::{DataType, Feature, FeatureId};
use crate::config::VWAPFeatureConfig;
use anyhow::{anyhow, Result};
use rust_decimal::prelude::*;
use std::{collections::HashMap, time::Duration};
use tracing::debug;

#[derive(Debug)]
pub struct VWAPFeature {
    id: FeatureId,
    trade_price: FeatureId,
    trade_quantity: FeatureId,
    window: Duration,
}

impl VWAPFeature {
    pub fn new(id: FeatureId, window: Duration) -> Self {
        VWAPFeature {
            id,
            trade_price: "trade_price".into(),
            trade_quantity: "trade_quantity".into(),
            window,
        }
    }

    pub fn from_config(config: &VWAPFeatureConfig) -> Self {
        VWAPFeature {
            id: config.id.to_owned(),
            trade_price: "trade_price".into(),
            trade_quantity: "trade_quantity".into(),
            window: Duration::from_secs(config.window),
        }
    }
}

impl Feature for VWAPFeature {
    fn id(&self) -> &FeatureId {
        &self.id
    }

    fn sources(&self) -> Vec<FeatureId> {
        vec![self.trade_price.clone(), self.trade_quantity.clone()]
    }

    fn data_type(&self) -> DataType {
        DataType::Window(self.window)
    }

    fn calculate(&self, data: HashMap<FeatureId, Vec<f64>>) -> Result<HashMap<FeatureId, f64>> {
        debug!("Calculating VWAP with id: {}", self.id);
        // Check if both trade_price and trade_quantity are present
        let price = data.get(&self.trade_price).ok_or(anyhow!("Missing trade_price"))?;
        let quantity = data.get(&self.trade_quantity).ok_or(anyhow!("Missing trade_quantity"))?;
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
