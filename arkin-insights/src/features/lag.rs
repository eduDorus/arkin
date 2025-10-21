use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use strum::Display;
use time::UtcDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{math::*, state::InsightsState, Feature};

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

    fn calculate(
        &self,
        state: &Arc<InsightsState>,
        pipeline: &Arc<Pipeline>,
        instrument: &Arc<Instrument>,
        event_time: UtcDateTime,
    ) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating {} change...", self.method);

        //  Get data
        let prev_value = state.lag(instrument, &self.input, event_time, self.lag);
        let value = state.last(instrument, &self.input, event_time);

        // Check if we have enough data
        if prev_value.is_none() || value.is_none() {
            warn!(
                "Not enough data for Change calculation: value {:?}, lag value {:?}",
                value, prev_value
            );
            return None;
        }

        // Unwrap values
        let prev_value = prev_value.expect("Prev value should not be None");
        let value = value.expect("Value should not be None");

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
