use std::collections::HashMap;

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::prelude::*;
use tracing::debug;

use crate::{
    config::StdDevFeatureConfig,
    service::FeatureModule,
    state::{DataRequest, DataResponse},
};

#[derive(Debug)]
pub struct StdDevFeature {
    id: NodeId,
    sources: Vec<NodeId>,
    inputs: Vec<DataRequest>,
    output: FeatureId,
}

impl StdDevFeature {
    pub fn from_config(config: &StdDevFeatureConfig) -> Self {
        let sources = vec![config.input.from.clone()];
        let data = vec![config.input.to_owned().into()];

        StdDevFeature {
            id: config.id.to_owned(),
            sources,
            inputs: data,
            output: config.output.to_owned(),
        }
    }
}

impl FeatureModule for StdDevFeature {
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
        debug!("Calculating Volatility with id: {}", self.id);

        let values = data.get(self.inputs[0].feature_id());

        // No values to calculate the volatility
        if values.is_empty() {
            return Ok(HashMap::new());
        }

        let mean = values.iter().sum::<Decimal>() / Decimal::from(values.len());
        let deviation = values.iter().map(|v| (v - mean)).collect::<Vec<_>>();
        let deviation_squared = deviation.iter().map(|v| v.powi(2)).collect::<Vec<_>>();
        let variance = deviation_squared.iter().sum::<Decimal>() / Decimal::from(deviation_squared.len());
        let standard_deviation = variance.sqrt().unwrap_or(Decimal::ZERO);

        let mut res = HashMap::new();
        res.insert(self.output.clone(), standard_deviation);
        Ok(res)
    }
}
