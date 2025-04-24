use std::sync::Arc;

use serde::Deserialize;
use strum::Display;
use time::OffsetDateTime;
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
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    input: FeatureId,
    output: FeatureId,
    lag: usize,
    method: LagAlgo,
    persist: bool,
}

impl Feature for LagFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, event_time: OffsetDateTime) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating {} change...", self.method);

        //  Get data
        let prev_value = self
            .insight_state
            .lag(Some(instrument.clone()), self.input.clone(), event_time, self.lag);
        let value = self
            .insight_state
            .last(Some(instrument.clone()), self.input.clone(), event_time);

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
        let insight = Insight::builder()
            .event_time(event_time)
            .pipeline(Some(self.pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(self.output.clone())
            .value(change)
            .insight_type(InsightType::Continuous)
            .persist(self.persist)
            .build()
            .into();

        Some(vec![insight])
    }
}
