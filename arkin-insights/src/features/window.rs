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
pub enum WindowMethod {
    // Basic
    Count,
    Sum,
    Mean,
    Median,
    Min,
    Max,
    Range,

    // Distribution
    Variance,
    StdDev,
    Skew,
    Kurtosis,
    Quantile(f64),
    Iqr,

    // Relationship
    Autocorrelation(usize),

    // Other
    VariationCoef,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct SlidingWindowFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    input: FeatureId,
    output: FeatureId,
    window: Duration,
    method: WindowMethod,
    persist: bool,
}

impl Feature for SlidingWindowFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, event_time: OffsetDateTime) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating {} distribution...", self.method);

        // Get data
        let data = self
            .insight_state
            .window(Some(instrument.clone()), self.input.clone(), event_time, self.window);

        // Check if we have enough data
        if data.len() < 2 {
            warn!("Not enough data for distribution calculation: {} entries", data.len());
            return None;
        }

        // Calculate distribution
        let value = match self.method {
            // Basic
            WindowMethod::Count => data.len() as f64,
            WindowMethod::Sum => sum(&data),
            WindowMethod::Mean => mean(&data),
            WindowMethod::Median => median(&data),
            WindowMethod::Min => min(&data),
            WindowMethod::Max => max(&data),
            WindowMethod::Range => range(&data),

            // Distribution
            WindowMethod::Variance => variance(&data),
            WindowMethod::StdDev => std_dev(&data),
            WindowMethod::Skew => skew(&data),
            WindowMethod::Kurtosis => kurtosis(&data),
            WindowMethod::Quantile(q) => quantile(&data, q),
            WindowMethod::Iqr => iqr(&data),

            // Relationship
            WindowMethod::Autocorrelation(lag) => autocorrelation(&data, lag),

            // Other
            WindowMethod::VariationCoef => variation_coef(&data),
        };

        // Check if we have a value
        if value == f64::NAN {
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
