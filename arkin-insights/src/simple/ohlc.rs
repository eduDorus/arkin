use std::{sync::Arc, time::Duration};

use anyhow::Result;
use time::OffsetDateTime;
use tracing::{debug, warn};

use arkin_core::prelude::*;

use crate::{config::OHLCConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct OHLCFeature {
    input: NodeId,
    open_output: NodeId,
    high_output: NodeId,
    low_output: NodeId,
    close_output: NodeId,
    window: Duration,
}

impl OHLCFeature {
    pub fn from_config(config: &OHLCConfig) -> Self {
        OHLCFeature {
            input: config.input.to_owned(),
            open_output: config.open_output.to_owned(),
            high_output: config.high_output.to_owned(),
            low_output: config.low_output.to_owned(),
            close_output: config.close_output.to_owned(),
            window: Duration::from_secs(config.window),
        }
    }
}

impl Computation for OHLCFeature {
    fn inputs(&self) -> Vec<NodeId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<NodeId> {
        vec![
            self.open_output.clone(),
            self.high_output.clone(),
            self.low_output.clone(),
            self.close_output.clone(),
        ]
    }

    fn calculate(
        &self,
        instruments: &[Instrument],
        timestamp: &OffsetDateTime,
        state: Arc<InsightsState>,
    ) -> Result<Vec<Insight>> {
        debug!("Calculating OHLC");

        // Get data from state
        let data = state.get_window_by_instruments(instruments, &self.input, timestamp, &self.window);

        // Calculate the mean (OHLC)
        let insights = data
            .into_iter()
            .filter_map(|(instrument, values)| {
                // Check if we have enough data
                if values.is_empty() {
                    warn!("Not enough data for OHLC calculation");
                    return None;
                }

                // Calculate OHLC
                let open = values.first().expect("Should have at least one value");
                let high = values.iter().max().expect("Should have at least one value");
                let low = values.iter().min().expect("Should have at least one value");
                let close = values.last().expect("Should have at least one value");

                // Create insights
                let open_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.open_output.clone(),
                    open.clone(),
                );
                let high_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.high_output.clone(),
                    high.clone(),
                );
                let low_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.low_output.clone(),
                    low.clone(),
                );
                let close_insight = Insight::new(
                    timestamp.clone(),
                    Some(instrument.clone()),
                    self.close_output.clone(),
                    close.clone(),
                );

                Some(vec![open_insight, high_insight, low_insight, close_insight])
            })
            .flatten()
            .collect::<Vec<_>>();

        state.insert_batch(insights.clone());
        Ok(insights)
    }
}
