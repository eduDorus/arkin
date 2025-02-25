use std::sync::Arc;

use anyhow::Result;
use rayon::prelude::*;
use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{state::InsightsState, Computation};

#[derive(Debug, Clone, TypedBuilder)]
pub struct SignalStrengthFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    input_first: FeatureId,
    input_second: FeatureId,
    output: FeatureId,
    persist: bool,
}

impl Computation for SignalStrengthFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input_first.clone(), self.input_second.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instruments: &[Arc<Instrument>], event_time: OffsetDateTime) -> Result<Vec<Arc<Insight>>> {
        debug!("Calculating Signal Strength...");

        // Calculate the mean (StdDev)
        let insights = instruments
            .par_iter()
            .filter_map(|instrument| {
                // Get data from state
                let first = self
                    .insight_state
                    .last(Some(instrument.clone()), self.input_first.clone(), event_time)?;

                let second =
                    self.insight_state
                        .last(Some(instrument.clone()), self.input_second.clone(), event_time)?;

                // Check if we don't have a total of 0
                if (first + second).is_zero() {
                    warn!("Total of 0 for Signal Strength calculation");
                    return None;
                }

                // Check that they are positive values
                if first < 0. || second < 0. {
                    warn!(
                        "Negative values for Signal Strength calculation: {} with values {} and {}",
                        self.output, first, second
                    );
                    return None;
                }

                let signal_strength = (first - second) / (first + second);

                Some(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(self.pipeline.clone())
                        .instrument(Some(instrument.clone()))
                        .feature_id(self.output.clone())
                        .value(signal_strength)
                        .persist(self.persist)
                        .build()
                        .into(),
                )
            })
            .collect::<Vec<_>>();

        self.insight_state.insert_batch(&insights);
        Ok(insights)
    }
}
