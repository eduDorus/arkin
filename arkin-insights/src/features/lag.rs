use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use strum::Display;
use time::UtcDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{math::*, Feature, FeatureStore, FillStrategy};

#[derive(Debug, Display, Clone, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LagAlgo {
    // Change
    AbsoluteChange,
    PercentChange,
    LogChange,
    Difference,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct LagFeature {
    input: FeatureId,
    output: FeatureId,
    lag: usize,
    method: LagAlgo,
    fill_strategy: FillStrategy,
    persist: bool,
}

#[async_trait]
impl Feature for LagFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn fill_strategy(&self) -> FillStrategy {
        self.fill_strategy
    }

    fn calculate(
        &self,
        state: &FeatureStore,
        pipeline: &Arc<Pipeline>,
        instrument: &Arc<Instrument>,
        event_time: UtcDateTime,
    ) -> Option<Vec<Arc<Insight>>> {
        debug!(target: "feature-calc", "Calculating {} for {} at {}", self.output, instrument, event_time);

        //  Get data - now returns Result
        let prev_value = match state.lag(instrument, &self.input, event_time, self.lag, Some(self.fill_strategy)) {
            Ok(v) => v,
            Err(e) => {
                warn!("Failed to get lagged value: {}", e);
                return None;
            }
        };

        let value = match state.last(instrument, &self.input, event_time) {
            Some(v) => v,
            None => {
                warn!("No current value available");
                return None;
            }
        };

        let mut change = match self.method {
            LagAlgo::AbsoluteChange => abs_change(value, prev_value),
            LagAlgo::PercentChange => pct_change(value, prev_value),
            LagAlgo::LogChange => log_change(value, prev_value),
            LagAlgo::Difference => difference(value, prev_value),
        };

        // Check if we have a value
        if change.is_nan() {
            warn!(
                "NaN value for distribution calculation for feature {} with method {}",
                self.output, self.method
            );
            return None;
        }

        // Set precision to 6 decimal places
        change = (change * 1_000_000.0).round() / 1_000_000.0;
        debug!(target: "feature-calc", "Calculated value for {}: {}", self.output, change);

        // Return insight
        let insight = vec![Arc::new(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(self.output.clone())
                .value(change)
                .insight_type(InsightType::Continuous)
                .persist(self.persist)
                .build(),
        )];

        // Save insight to state
        state.insert_batch(insight.as_slice());

        Some(insight)
    }

    // async fn async_calculate(&self, instrument: &Arc<Instrument>, timestamp: UtcDateTime) -> Option<Vec<Insight>> {
    //     self.calculate(instrument, timestamp)
    // }
}
