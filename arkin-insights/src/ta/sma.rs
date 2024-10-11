use std::sync::Arc;

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};

use arkin_core::prelude::*;

use crate::{config::SMAConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct SMAFeature {
    input: NodeId,
    output: NodeId,
    periods: usize,
}

impl SMAFeature {
    pub fn from_config(config: &SMAConfig) -> Self {
        SMAFeature {
            input: config.input.to_owned(),
            output: config.output.to_owned(),
            periods: config.periods,
        }
    }
}

impl Computation for SMAFeature {
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

        // Get data from state
        let data = state.get_periods_by_instruments(instruments, &self.input, timestamp, &self.periods);

        // Calculate the mean (SMA)
        let insights = data
            .into_iter()
            .filter_map(|(instrument, values)| {
                // Check if we have enough data
                if values.len() < self.periods {
                    warn!("Not enough data for SMA calculation");
                    return None;
                }

                // Calculate SMA
                let count = Decimal::from(values.len());
                let sum = values.iter().sum::<Decimal>();
                let sma = sum / count;

                Some(Insight::new(timestamp.clone(), Some(instrument), self.output.clone(), sma))
            })
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
