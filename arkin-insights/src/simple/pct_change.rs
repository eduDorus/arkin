use std::sync::Arc;

use anyhow::Result;
use arkin_core::prelude::*;
use time::OffsetDateTime;
use tracing::debug;

use crate::{config::PctChangeConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct PctChangeFeature {
    input: NodeId,
    output: NodeId,
    periods: usize,
}

impl PctChangeFeature {
    pub fn from_config(config: &PctChangeConfig) -> Self {
        PctChangeFeature {
            input: config.input.to_owned(),
            output: config.output.to_owned(),
            periods: config.periods,
        }
    }
}

impl Computation for PctChangeFeature {
    fn inputs(&self) -> Vec<NodeId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<NodeId> {
        vec![self.output.clone()]
    }

    fn calculate(
        &self,
        instruments: &[Instrument],
        timestamp: &OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>> {
        debug!("Calculating percentage change");

        // Get data from state

        // Retrieve the values for the feature over the window period
        let insights = instruments
            .iter()
            .filter_map(|instrument| {
                //  Get data
                let data = state.get_periods(Some(instrument), &self.input, timestamp, &self.periods);

                // Check if we have enough data
                if data.len() < 2 {
                    return None;
                }

                // Calculate the percentage change
                let first_value = data
                    .first()
                    .expect("Could not get first value, unexpected empty vector, should have been caught earlier");
                let last_value = data
                    .last()
                    .expect("Could not get last value, unexpected empty vector, should have been caught earlier");
                let percentage_change = (last_value - first_value) / first_value;

                // Return insight
                Some(Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.output.clone(),
                    percentage_change,
                ))
            })
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
