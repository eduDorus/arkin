use std::sync::Arc;

use anyhow::Result;
use arkin_core::prelude::*;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::debug;
use uuid::Uuid;

use crate::{config::HistVolConfig, state::InsightsState, Computation};

#[derive(Debug)]
pub struct HistVolFeature {
    input: FeatureId,
    output: FeatureId,
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
                // Get data
                let data = state.last(Some(instrument.clone()), self.input.clone(), timestamp);

                // Check if we have enough data
                if data.is_none() {
                    return None;
                }

                // Calculate the annualized volatility
                let v = data.expect("Could not get value, unexpected None, should have been caught earlier");
                let annualized_vol = v * self.annualized_multi;

                // Create the insight
                Some(
                    Insight::builder()
                        .id(Uuid::new_v4())
                        .pipeline(pipeline)
                        .event_time(timestamp)
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output.clone())
                        .value(annualized_vol)
                        .build(),
                )
            })
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
