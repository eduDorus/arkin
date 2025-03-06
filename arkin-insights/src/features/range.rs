use std::{sync::Arc, time::Duration};

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
pub enum RangeData {
    Window(u64),
    Interval(usize),
}

#[derive(Debug, Display, Clone, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RangeAlgo {
    // Basic
    Count,
    Sum,
    Mean,
    Median,
    Min,
    Max,
    AbsolutRange,
    RelativeRange,
    RelativePosition,

    // Distribution
    Variance,
    StandardDeviation,
    Skew,
    Kurtosis,
    Quantile(f64),
    Iqr,

    // Relationship
    Autocorrelation(usize),

    // Other
    CoefOfVariation,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct RangeFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    input: FeatureId,
    output: FeatureId,
    method: RangeAlgo,
    data: RangeData,
    persist: bool,
}

impl Feature for RangeFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, event_time: OffsetDateTime) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating {}...", self.method);

        // Get data
        let data = match self.data {
            RangeData::Interval(i) => {
                self.insight_state
                    .intervals(Some(instrument.clone()), self.input.clone(), event_time, i)
            }
            RangeData::Window(w) => self.insight_state.window(
                Some(instrument.clone()),
                self.input.clone(),
                event_time,
                Duration::from_secs(w),
            ),
        };

        // Check if we have enough data
        if data.len() < 2 {
            warn!("Not enough data for distribution calculation: {} entries", data.len());
            return None;
        }

        // Calculate distribution
        let value = match self.method {
            // Basic
            RangeAlgo::Count => data.len() as f64,
            RangeAlgo::Sum => sum(&data),
            RangeAlgo::Mean => mean(&data),
            RangeAlgo::Median => median(&data),
            RangeAlgo::Min => min(&data),
            RangeAlgo::Max => max(&data),
            RangeAlgo::AbsolutRange => absolut_range(&data),
            RangeAlgo::RelativeRange => relative_range(&data),
            RangeAlgo::RelativePosition => relative_position(&data),

            // Distribution
            RangeAlgo::Variance => variance(&data),
            RangeAlgo::StandardDeviation => std_dev(&data),
            RangeAlgo::Skew => skew(&data),
            RangeAlgo::Kurtosis => kurtosis(&data),
            RangeAlgo::Quantile(q) => quantile(&data, q),
            RangeAlgo::Iqr => iqr(&data),

            // Relationship
            RangeAlgo::Autocorrelation(lag) => autocorrelation(&data, lag),

            // Other
            RangeAlgo::CoefOfVariation => coefficient_of_variation(&data),
        };

        // Check if we have a value
        if value.is_nan() {
            warn!(
                "NaN value for distribution calculation for feature {} with method {}",
                self.output, self.method
            );
            return None;
        }

        let insight = Insight::builder()
            .event_time(event_time)
            .pipeline(Some(self.pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(self.output.clone())
            .value(value)
            .insight_type(InsightType::Continuous)
            .persist(self.persist)
            .build()
            .into();

        Some(vec![insight])
    }
}
