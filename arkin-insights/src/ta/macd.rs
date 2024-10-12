use std::sync::Arc;

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};

use arkin_core::prelude::*;

use crate::{config::MACDConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct MACDFeature {
    fast_input: NodeId,
    slow_input: NodeId,
    signal_output: NodeId,
    histogram_output: NodeId,
    smoothing_constant: Decimal,
}

impl MACDFeature {
    pub fn from_config(config: &MACDConfig) -> Self {
        let smoothing_constant =
            Decimal::from(config.smoothing) / (Decimal::from(config.signal_periods) + Decimal::from(1));
        MACDFeature {
            fast_input: config.fast_input.to_owned(),
            slow_input: config.slow_input.to_owned(),
            signal_output: config.signal_output.to_owned(),
            histogram_output: config.histogram_output.to_owned(),
            smoothing_constant,
        }
    }
}

impl Computation for MACDFeature {
    fn inputs(&self) -> Vec<NodeId> {
        vec![self.fast_input.clone(), self.slow_input.clone()]
    }

    fn outputs(&self) -> Vec<NodeId> {
        vec![self.histogram_output.clone(), self.signal_output.clone()]
    }

    fn calculate(
        &self,
        instruments: &[Instrument],
        timestamp: &OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>> {
        debug!("Calculating MACD");

        // Calculate the mean (MACD)
        let insights = instruments
            .iter()
            .filter_map(|instrument| {
                // Get data from state
                let fast_data = state.get_last_by_instrument(Some(instrument), &self.fast_input, timestamp);
                let slow_data = state.get_last_by_instrument(Some(instrument), &self.slow_input, timestamp);

                // Check if we have enough data
                if fast_data.is_none() || slow_data.is_none() {
                    warn!("Not enough data for MACD calculation");
                    return None;
                }

                let fast = fast_data.expect("Fast is None there but was checked before");
                let slow = slow_data.expect("Slow is None but was checked before");

                // Calculate MACD
                let macd_line = fast - slow;

                // Calculate EMA of MACD line
                let prev_signal = state.get_last_by_instrument(Some(instrument), &self.signal_output, timestamp);
                let signal_line = match prev_signal {
                    Some(s) => (macd_line - s) * self.smoothing_constant + s,
                    None => macd_line,
                };

                let signal_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.signal_output.clone(),
                    signal_line,
                );

                let histogram = macd_line - signal_line;
                let histogram_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.histogram_output.clone(),
                    histogram,
                );
                Some(vec![signal_insight, histogram_insight])
            })
            .flatten()
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
