use std::sync::Arc;

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};

use arkin_core::prelude::*;

use crate::{config::EMAConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct EMAFeature {
    input: NodeId,
    output: NodeId,
    periods: usize,
    smoothing_constant: Decimal,
}

impl EMAFeature {
    pub fn from_config(config: &EMAConfig) -> Self {
        let smoothing_constant = Decimal::from(config.smoothing) / (Decimal::from(config.periods) + Decimal::from(1));
        EMAFeature {
            input: config.input.to_owned(),
            output: config.output.to_owned(),
            periods: config.periods,
            smoothing_constant,
        }
    }
}

impl Computation for EMAFeature {
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
        debug!("Calculating EMA");

        // Get data from state
        let data = state.get_periods_by_instruments(instruments, &self.input, timestamp, &self.periods);
        let last_emas = state.get_last_by_instruments(instruments, &self.output, timestamp);

        // Calculate the mean (EMA)
        let insights = data
            .into_iter()
            .filter_map(|(instrument, values)| {
                // Check if we have enough data
                if values.len() < self.periods {
                    warn!("Not enough data for EMA calculation");
                    return None;
                }

                // Check if the instrument has an EMA entry
                match last_emas.get(&instrument) {
                    // If key exists and has a last EMA value, proceed with the calculation
                    Some(Some(last_ema)) => {
                        // Calculate SMA
                        let sum = values.iter().sum::<Decimal>();
                        let count = Decimal::from(values.len());
                        let sma = sum / count;

                        // Calculate EMA
                        let ema =
                            sma * self.smoothing_constant + last_ema * (Decimal::from(1) - self.smoothing_constant);
                        Some(Insight::new(timestamp.clone(), Some(instrument), self.output.clone(), ema))
                    }
                    // If key exists but has no last EMA value, use SMA as the starting EMA
                    Some(None) => {
                        warn!("Instrument {:?} has no last EMA value, using SMA as fallback", instrument);
                        let sum = values.iter().sum::<Decimal>();
                        let count = Decimal::from(values.len());
                        let sma = sum / count;
                        Some(Insight::new(timestamp.clone(), Some(instrument), self.output.clone(), sma))
                    }
                    // If key does not exist, skip this instrument
                    None => {
                        warn!("No EMA data found for instrument {:?}", instrument);
                        None
                    }
                }
            })
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
