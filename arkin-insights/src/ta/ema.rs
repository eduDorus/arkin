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
    multiplier: Decimal,
}

impl EMAFeature {
    pub fn from_config(config: &EMAConfig) -> Self {
        let multiplier = Decimal::from(config.smoothing) / (Decimal::from(config.periods) + Decimal::from(1));
        EMAFeature {
            input: config.input.to_owned(),
            output: config.output.to_owned(),
            periods: config.periods,
            multiplier,
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

                // Calculate SMA
                let sum = values.iter().sum::<Decimal>();
                let count = Decimal::from(values.len());
                let sma = sum / count;

                // Calculate the EMA (EMA = last value x multiplier + EMA (previous day) x (1-multiplier))
                let last_ema = last_emas.get(&instrument).unwrap_or(&sma);
                let ema = sma * self.multiplier + last_ema * (Decimal::from(1) - self.multiplier);

                Some(Insight::new(timestamp.clone(), Some(instrument), self.output.clone(), ema))
            })
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
