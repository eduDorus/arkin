use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use serde::Deserialize;
use strum::Display;
use time::UtcDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{math::*, Feature, FeatureState};

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
    SumPositive,
    SumNegative,
    SumAbs,
    SumAbsPositive,
    SumAbsNegative,
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
    input: FeatureId,
    output: FeatureId,
    method: RangeAlgo,
    data: RangeData,
    persist: bool,
}

#[async_trait]
impl Feature for RangeFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(
        &self,
        state: &FeatureState,
        pipeline: &Arc<Pipeline>,
        instrument: &Arc<Instrument>,
        event_time: UtcDateTime,
    ) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating {}...", self.method);

        // Get data
        let data = match self.data {
            RangeData::Interval(i) => state.last_n(instrument, &self.input, event_time, i),
            RangeData::Window(w) => state.window(instrument, &self.input, event_time, Duration::from_secs(w)),
        };

        // Check if we have enough data
        if data.len() < 2 {
            warn!("Not enough data for distribution calculation: {} entries", data.len());
            return None;
        }

        // Calculate distribution
        let mut value = match self.method {
            // Basic
            RangeAlgo::Count => data.len() as f64,
            RangeAlgo::Sum => sum(&data),
            RangeAlgo::SumAbs => sum_abs(&data),
            RangeAlgo::SumPositive => sum_positive(&data),
            RangeAlgo::SumNegative => sum_negative(&data),
            RangeAlgo::SumAbsPositive => sum_abs_positive(&data),
            RangeAlgo::SumAbsNegative => sum_abs_negative(&data),
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

        // Set precision to 6 decimal places
        value = (value * 1_000_000.0).round() / 1_000_000.0;

        let insight = vec![Arc::new(
            Insight::builder()
                .event_time(event_time)
                .pipeline(Some(pipeline.clone()))
                .instrument(instrument.clone())
                .feature_id(self.output.clone())
                .value(value)
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
