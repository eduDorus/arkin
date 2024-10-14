use std::sync::Arc;

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};

use arkin_core::prelude::*;

use crate::{config::SMAConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct SimpleMovingAverageFeature {
    input: NodeId,
    output: NodeId,
    periods: usize,
}

impl SimpleMovingAverageFeature {
    pub fn from_config(config: &SMAConfig) -> Self {
        SimpleMovingAverageFeature {
            input: config.input.to_owned(),
            output: config.output.to_owned(),
            periods: config.periods,
        }
    }
}

impl Computation for SimpleMovingAverageFeature {
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
        debug!("Calculating SMA");

        // Calculate the mean (SMA)
        let insights = instruments
            .iter()
            .filter_map(|instrument| {
                // Get data from state
                let values = state.get_periods(Some(instrument), &self.input, timestamp, &self.periods);

                // Check if we have enough data
                if values.len() < self.periods {
                    warn!("Not enough data for SMA calculation");
                    return None;
                }

                // Calculate SMA
                let count = Decimal::from(values.len());
                let sum = values.iter().sum::<Decimal>();
                let sma = sum / count;

                Some(Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.output.clone(),
                    sma,
                ))
            })
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
