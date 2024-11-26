use std::sync::Arc;

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};

use arkin_core::prelude::*;

use crate::{config::EMAConfig, state::InsightsState, Computation};

#[derive(Debug)]
pub struct ExponentialMovingAverageFeature {
    input: FeatureId,
    output: FeatureId,
    periods: usize,
    smoothing_constant: Decimal,
}

impl ExponentialMovingAverageFeature {
    pub fn from_config(config: &EMAConfig) -> Self {
        let smoothing_constant = Decimal::from(config.smoothing) / (Decimal::from(config.periods) + Decimal::from(1));
        ExponentialMovingAverageFeature {
            input: config.input.to_owned(),
            output: config.output.to_owned(),
            periods: config.periods,
            smoothing_constant,
        }
    }
}

impl Computation for ExponentialMovingAverageFeature {
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
        debug!("Calculating EMA");

        // Calculate the mean (EMA)
        let insights = instruments
            .iter()
            .cloned()
            .filter_map(|instrument| {
                // Get data
                let values = state.periods(Some(instrument.clone()), self.input.clone(), timestamp, self.periods);

                // Check if we have enough data
                if values.len() < self.periods {
                    warn!("Not enough data for EMA calculation");
                    return None;
                }

                // Check if the instrument has an EMA entry
                let last_ema = state.last(Some(instrument.clone()), self.output.clone(), timestamp);
                match last_ema {
                    // If key exists and has a last EMA value, proceed with the calculation
                    Some(last_ema) => {
                        // Calculate SMA
                        let sum = values.iter().sum::<Decimal>();
                        let count = Decimal::from(values.len());
                        let sma = sum / count;

                        // Calculate EMA
                        let ema =
                            sma * self.smoothing_constant + last_ema * (Decimal::from(1) - self.smoothing_constant);
                        Some(Insight::new(timestamp, Some(instrument.clone()), self.output.clone(), ema))
                    }
                    // If key exists but has no last EMA value, use SMA as the starting EMA
                    None => {
                        warn!("Instrument {} has no last EMA value, using SMA as fallback", instrument);
                        let sum = values.iter().sum::<Decimal>();
                        let count = Decimal::from(values.len());
                        let sma = sum / count;
                        Some(Insight::new(timestamp, Some(instrument), self.output.clone(), sma))
                    }
                }
            })
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
