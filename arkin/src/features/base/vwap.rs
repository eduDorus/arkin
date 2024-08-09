use crate::config::VWAPFeatureConfig;
use crate::features::{Feature, FeatureDataRequest, FeatureDataResponse, FeatureId, NodeId};
use anyhow::Result;
use rust_decimal::prelude::*;
use std::collections::HashMap;
use tracing::debug;

#[derive(Debug)]
pub struct VWAPFeature {
    id: NodeId,
    sources: Vec<NodeId>,
    inputs: Vec<FeatureDataRequest>,
    output: FeatureId,
}

impl VWAPFeature {
    pub fn from_config(config: &VWAPFeatureConfig) -> Self {
        VWAPFeature {
            id: config.id.to_owned(),
            sources: vec![config.input_price.from.clone(), config.input_quantity.from.clone()],
            inputs: vec![config.input_price.to_owned().into(), config.input_quantity.to_owned().into()],
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
        &self.inputs
    }

    fn calculate(&self, data: FeatureDataResponse) -> Result<HashMap<FeatureId, f64>> {
        debug!("Calculating VWAP with id: {}", self.id);
        // Check if both trade_price and trade_quantity are present
        let price = data.get(self.inputs[0].feature_id());
        let quantity = data.get(self.inputs[1].feature_id());
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
