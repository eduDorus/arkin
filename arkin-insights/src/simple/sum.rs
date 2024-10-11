use std::sync::Arc;

use anyhow::Result;
use arkin_core::prelude::*;
use time::OffsetDateTime;
use tracing::debug;

use crate::{config::SumFeatureConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct SumFeature {
    inputs: Vec<NodeId>,
    outputs: Vec<NodeId>,
}

impl SumFeature {
    pub fn from_config(config: &SumFeatureConfig) -> Self {
        Self {
            inputs: config.inputs.to_owned(),
            outputs: config.outputs.to_owned(),
        }
    }
}

impl Computation for SumFeature {
    fn inputs(&self) -> &[NodeId] {
        &self.inputs
    }

    fn outputs(&self) -> &[NodeId] {
        &self.outputs
    }

    fn calculate(
        &self,
        instruments: &[Instrument],
        timestamp: &OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>> {
        debug!("Calculating Sum with id: {}", self.inputs[0]);

        // Query the data from the internal state
        let data = self
            .inputs()
            .into_iter()
            .zip(self.outputs())
            .map(|(input_id, output_id)| {
                let values = instruments
                    .iter()
                    .filter_map(|instrument| state.get_last_by_instrument(Some(instrument), input_id, timestamp))
                    .collect::<Vec<_>>();
                let sum = values.iter().sum();
                Insight::new(timestamp.clone(), None, output_id.clone(), sum)
            })
            .collect::<Vec<_>>();

        state.insert_batch(data.clone());
        Ok(data)
    }
}
