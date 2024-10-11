use std::{sync::Arc, time::Duration};

use anyhow::Result;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};

use arkin_core::prelude::*;

use crate::{config::StdDevConfig, service::Computation, state::InsightsState};

#[derive(Debug)]
pub struct StdDevFeature {
    input: NodeId,
    output: NodeId,
    window: Duration,
}

impl StdDevFeature {
    pub fn from_config(config: &StdDevConfig) -> Self {
        StdDevFeature {
            input: config.input.to_owned(),
            output: config.output.to_owned(),
            window: Duration::from_secs(config.window),
        }
    }
}

impl Computation for StdDevFeature {
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
        debug!("Calculating StdDev");

        // Get data from state
        let data = state.get_window_by_instruments(instruments, &self.input, timestamp, &self.window);

        // Calculate the mean (StdDev)
        let insights = data
            .into_iter()
            .filter_map(|(instrument, values)| {
                // Check if we have enough data
                if values.is_empty() {
                    warn!("Not enough data for StdDev calculation");
                    return None;
                }

                // Calculate StdDev
                let sum = values.iter().sum::<Decimal>();
                let count = Decimal::from(values.len());
                let mean = sum / count;
                let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<Decimal>() / count;
                if let Some(std_dev) = variance.sqrt() {
                    Some(Insight::new(timestamp.clone(), Some(instrument), self.output.clone(), std_dev))
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
