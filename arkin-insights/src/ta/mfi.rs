use std::sync::Arc;

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, info, warn};

use arkin_core::prelude::*;

use crate::{config::RelativeStrengthIndexConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct RSIFeature {
    input_return: FeatureId,
    output: FeatureId,
    periods: usize,
}

impl RSIFeature {
    pub fn from_config(config: &RelativeStrengthIndexConfig) -> Self {
        RSIFeature {
            input_return: config.input_return.to_owned(),
            output: config.output.to_owned(),
            periods: config.periods,
        }
    }
}

impl Computation for RSIFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input_return.clone()]
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
        debug!("Calculating RSI");

        // Calculate the mean (RSI)
        let insights = instruments
            .iter()
            .filter_map(|instrument| {
                // Get data
                let returns = state.get_periods(Some(instrument), &self.input_return, timestamp, &(self.periods + 1));

                // Check if we have enough data
                if returns.len() < self.periods + 1 {
                    warn!("Not enough data for RSI calculation");
                    return None;
                }

                // Separate gains and losses
                let (mut gains, mut losses) =
                    returns.iter().fold((Vec::new(), Vec::new()), |(mut gains, mut losses), r| {
                        if *r > Decimal::ZERO {
                            gains.push(*r);
                        } else {
                            losses.push(r.abs());
                        }
                        (gains, losses)
                    });
                let last_gain = gains.pop().unwrap_or(Decimal::ZERO);
                let last_loss = losses.pop().unwrap_or(Decimal::ZERO);
                let prev_avg_gain = gains.iter().sum::<Decimal>() / Decimal::from(self.periods);
                let prev_avg_loss = losses.iter().sum::<Decimal>() / Decimal::from(self.periods);

                // Calculate the RSI
                let rsi_gain = prev_avg_gain * Decimal::from(self.periods - 1) + last_gain;
                let rsi_loss = prev_avg_loss * Decimal::from(self.periods - 1) + last_loss;

                // Zero loss edge case
                if rsi_loss.is_zero() {
                    return Some(Insight::new(
                        timestamp,
                        Some(instrument.clone()),
                        self.output.clone(),
                        Decimal::from(100),
                    ));
                } else {
                    let ratio = rsi_gain / rsi_loss;
                    let rsi = Decimal::from(100) - (Decimal::from(100) / (Decimal::from(1) + ratio));
                    return Some(Insight::new(
                        timestamp,
                        Some(instrument.clone()),
                        self.output.clone(),
                        rsi,
                    ));
                }
            })
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
