use std::sync::Arc;

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};

use arkin_core::prelude::*;

use crate::{config::StdDevConfig, state::InsightsState, Computation};

#[derive(Debug)]
pub struct StdDevFeature {
    input: FeatureId,
    output: FeatureId,
    periods: usize,
}

impl StdDevFeature {
    pub fn from_config(config: &StdDevConfig) -> Self {
        StdDevFeature {
            input: config.input.to_owned(),
            output: config.output.to_owned(),
            periods: config.periods,
        }
    }
}

impl Computation for StdDevFeature {
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
        debug!("Calculating StdDev");

        // Calculate the mean (StdDev)
        let insights = instruments
            .iter()
            .cloned()
            .filter_map(|instrument| {
                // Get data from state
                let data = state.periods(Some(instrument.clone()), self.input.clone(), timestamp, self.periods);

                // Check if we have enough data
                if data.is_empty() {
                    warn!("Not enough data for StdDev calculation");
                    return None;
                }

                // Calculate StdDev
                let sum = data.iter().sum::<Decimal>();
                let count = Decimal::from(data.len());
                let mean = sum / count;
                let variance = data.iter().map(|v| (v - mean).powi(2)).sum::<Decimal>() / count;
                if let Some(std_dev) = variance.sqrt() {
                    Some(Insight::new(timestamp, Some(instrument.clone()), self.output.clone(), std_dev))
                } else {
                    warn!("Failed to calculate StdDev: mean: {}, variance: {}", mean, variance);
                    None
                }
            })
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
