use std::sync::Arc;

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, info, warn};

use arkin_core::prelude::*;

use crate::{config::PctChangeConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct PctChangeFeature {
    input: FeatureId,
    output: FeatureId,
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
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(
        &self,
        instruments: &[Arc<Instrument>],
        timestamp: OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>> {
        debug!("Calculating percentage change");

        // Get data from state

        // Retrieve the values for the feature over the window period
        let insights = instruments
            .iter()
            .filter_map(|instrument| {
                //  Get data
                info!("Getting data for timestamp: {:?}", timestamp);
                let data = state.periods(Some(instrument.clone()), self.input.clone(), timestamp, self.periods + 1);

                // Check if we have enough data
                if data.len() < self.periods + 1 {
                    warn!("Not enough data to calculate percent change");
                    return None;
                }
                info!("Data: {:?}", data);

                // Calculate the percentage change
                let first_value = data
                    .first()
                    .expect("Could not get first value, unexpected empty vector, should have been caught earlier");
                let last_value = data
                    .last()
                    .expect("Could not get last value, unexpected empty vector, should have been caught earlier");
                let difference = last_value - first_value;
                let avg = (first_value + last_value) / Decimal::from(2);
                let percentage_change = difference / avg;
                let percentage_change = percentage_change * Decimal::from(100);

                // Return insight
                Some(Insight::new(
                    timestamp,
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
