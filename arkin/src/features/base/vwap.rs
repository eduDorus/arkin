use crate::config::VWAPFeatureConfig;
use crate::features::{Feature, FeatureDataRequest, FeatureDataResponse, FeatureId, NodeId, Window};
use anyhow::Result;
use rust_decimal::prelude::*;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct VWAPFeature {
    id: NodeId,
    sources: Vec<NodeId>,
    data: Vec<FeatureDataRequest>,
    input_price: Window,
    input_quantity: Window,
    output: FeatureId,
}

impl VWAPFeature {
    pub fn from_config(config: &VWAPFeatureConfig) -> Self {
        let mut sources = vec![config.input_price.from.clone(), config.input_quantity.from.clone()];
        sources.dedup();
        let data = vec![config.input_price.to_owned().into(), config.input_quantity.to_owned().into()];
        VWAPFeature {
            id: config.id.to_owned(),
            sources,
            data,
            input_price: config.input_price.to_owned(),
            input_quantity: config.input_quantity.to_owned(),
            output: config.output.to_owned(),
        }
    }
}

impl Feature for VWAPFeature {
    fn id(&self) -> &NodeId {
        &self.id
    }

    fn sources(&self) -> &[NodeId] {
        &self.sources
    }

    fn data(&self) -> &[FeatureDataRequest] {
        &self.data
    }

    fn calculate(&self, data: FeatureDataResponse) -> Result<HashMap<FeatureId, f64>> {
        debug!("Calculating VWAP with id: {}", self.id);
        // Check if both trade_price and trade_quantity are present
        let price = data.get(&self.input_price.feature_id);
        let quantity = data.get(&self.input_quantity.feature_id);
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
        res.insert(self.output.clone(), vwap);
        Ok(res)
    }
}
