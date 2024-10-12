use std::sync::Arc;

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};

use arkin_core::prelude::*;

use crate::{config::MACDConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct MACDFeature {
    input_fast: NodeId,
    input_slow: NodeId,
    output_signal: NodeId,
    output_histogram: NodeId,
    smoothing_constant: Decimal,
}

impl MACDFeature {
    pub fn from_config(config: &MACDConfig) -> Self {
        let smoothing_constant =
            Decimal::from(config.smoothing) / (Decimal::from(config.signal_periods) + Decimal::from(1));
        MACDFeature {
            input_fast: config.input_fast.to_owned(),
            input_slow: config.input_slow.to_owned(),
            output_signal: config.output_signal.to_owned(),
            output_histogram: config.output_histogram.to_owned(),
            smoothing_constant,
        }
    }
}

impl Computation for MACDFeature {
    fn inputs(&self) -> Vec<NodeId> {
        vec![self.input_fast.clone(), self.input_slow.clone()]
    }

    fn outputs(&self) -> Vec<NodeId> {
        vec![self.output_histogram.clone(), self.output_signal.clone()]
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
                let fast_data = state.get_last_by_instrument(Some(instrument), &self.input_fast, timestamp);
                let slow_data = state.get_last_by_instrument(Some(instrument), &self.input_slow, timestamp);

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
                let prev_signal = state.get_last_by_instrument(Some(instrument), &self.output_signal, timestamp);
                let signal_line = match prev_signal {
                    Some(s) => (macd_line - s) * self.smoothing_constant + s,
                    None => macd_line,
                };

                let signal_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.output_signal.clone(),
                    signal_line,
                );

                let histogram = macd_line - signal_line;
                let histogram_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.output_histogram.clone(),
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
