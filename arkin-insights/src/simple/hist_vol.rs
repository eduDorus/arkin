use std::sync::Arc;

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::debug;

use crate::{config::HistVolConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct HistVolFeature {
    input: NodeId,
    output: NodeId,
    annualized_multi: Decimal,
}

impl HistVolFeature {
    pub fn from_config(config: &HistVolConfig) -> Self {
        // Calculate annualized multiplier (seconds)
        let multiplier =
            config.trading_days_per_year * Decimal::from_f64(24.0 * 60.0 * 60.0).unwrap() / config.timeframe_in_secs;
        let annualized_multi = multiplier.sqrt().expect("Failed to calculate annualization multiplier");
        HistVolFeature {
            input: config.input.to_owned(),
            output: config.output.to_owned(),
            annualized_multi,
        }
    }
}

impl Computation for HistVolFeature {
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
        let data = state.get_last_by_instruments(instruments, &self.input, timestamp);

        // Retrieve the values for the feature over the window period
        let insights = data
            .into_iter()
            .filter_map(|(i, v)| {
                // Check if we have enough data
                if v.is_none() {
                    return None;
                }

                let v = v.expect("Could not get value, unexpected None, should have been caught earlier");
                let annualized_vol = v * self.annualized_multi;

                Some(Insight::new(timestamp.clone(), Some(i), self.output.clone(), annualized_vol))
            })
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
