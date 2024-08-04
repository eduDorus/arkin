use super::{Feature, FeatureID};
use crate::config::VWAPFeatureConfig;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use rust_decimal::prelude::*;
use std::{collections::HashMap, time::Duration};
use tracing::info;

#[derive(Debug)]
pub struct VWAPFeature {
    id: FeatureID,
    trade_price: FeatureID,
    trade_quantity: FeatureID,
    _window: Duration,
}

impl VWAPFeature {
    pub fn new(id: FeatureID, window: Duration) -> Self {
        VWAPFeature {
            id,
            trade_price: "trade_price".into(),
            trade_quantity: "trade_quantity".into(),
            _window: window,
        }
    }

    pub fn from_config(config: &VWAPFeatureConfig) -> Self {
        VWAPFeature {
            id: config.id.to_owned(),
            trade_price: "trade_price".into(),
            trade_quantity: "trade_quantity".into(),
            _window: Duration::from_secs(config.window),
        }
    }
}

#[async_trait]
impl Feature for VWAPFeature {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<&FeatureID> {
        vec![&self.trade_price, &self.trade_quantity]
    }

    fn calculate(&self, data: HashMap<FeatureID, Vec<f64>>) -> Result<HashMap<FeatureID, f64>> {
        info!("Calculating VWAP with id: {}", self.id);
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
