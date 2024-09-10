use std::collections::HashMap;

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::Decimal;
use tracing::debug;

use crate::{
    config::VWAPFeatureConfig,
    manager::FeatureModule,
    state::{DataRequest, DataResponse},
};

#[derive(Debug)]
pub struct VWAPFeature {
    id: NodeId,
    sources: Vec<NodeId>,
    inputs: Vec<DataRequest>,
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

impl FeatureModule for VWAPFeature {
    fn id(&self) -> &NodeId {
        &self.id
    }

    fn sources(&self) -> &[NodeId] {
        &self.sources
    }

    fn data(&self) -> &[DataRequest] {
        &self.inputs
    }

    fn calculate(&self, data: DataResponse) -> Result<HashMap<FeatureId, Decimal>> {
        debug!("Calculating VWAP with id: {}", self.id);

        // Retrieve prices and volumes separately
        let prices = data.get(self.inputs[0].feature_id());
        let volumes = data.get(self.inputs[1].feature_id());

        // Calculate the sum of price * volume and the sum of volume
        let (total_price_volume, total_volume) = prices.iter().zip(volumes.iter()).fold(
            (Decimal::ZERO, Decimal::ZERO),
            |(acc_price_volume, acc_volume), (price, volume)| {
                (acc_price_volume + (price * volume), acc_volume + volume)
            },
        );

        // Calculate the VWAP
        let vwap = if total_volume.is_zero() {
            Decimal::ZERO
        } else {
            total_price_volume / total_volume
        };

        let mut res = HashMap::new();
        res.insert(self.output.clone(), vwap);
        Ok(res)
    }
}
