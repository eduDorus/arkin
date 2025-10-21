use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use serde::Deserialize;
use strum::Display;
use time::UtcDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{features::RangeData, math::*, Feature, FeatureState};

#[derive(Debug, Display, Clone, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DualRangeAlgo {
    Covariance,
    Correlation,
    CosineSimilarity,
    Beta,
    WeightedMean,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct DualRangeFeature {
    input_1: FeatureId,
    input_2: FeatureId,
    output: FeatureId,
    method: DualRangeAlgo,
    data: RangeData,
    persist: bool,
}

#[async_trait]
impl Feature for DualRangeFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input_1.clone(), self.input_2.clone()]
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
        let data_1 = match self.data {
            RangeData::Interval(i) => state.last_n(instrument, &self.input_1, event_time, i),
            RangeData::Window(w) => state.window(instrument, &self.input_1, event_time, Duration::from_secs(w)),
        };
        let data_2 = match self.data {
            RangeData::Interval(i) => state.last_n(instrument, &self.input_2, event_time, i),
            RangeData::Window(w) => state.window(instrument, &self.input_2, event_time, Duration::from_secs(w)),
        };

        // Check if we have enough data
        if data_1.len() < 2 || data_2.len() < 2 {
            warn!(
                "Not enough data for {} calculation: input_1 {}, input_2 {}",
                self.output,
                data_1.len(),
                data_2.len()
            );
            return None;
        }

        // Check that they have the same length
        if data_1.len() != data_2.len() {
            warn!(
                "Data length mismatch for {} calculation: input_1 {}, input_2 {}",
                self.output,
                data_1.len(),
                data_2.len()
            );
            return None;
        }

        // Calculate distribution
        let mut value = match self.method {
            DualRangeAlgo::Covariance => covariance(&data_1, &data_2),
            DualRangeAlgo::Correlation => correlation(&data_1, &data_2),
            DualRangeAlgo::CosineSimilarity => cosine_similarity(&data_1, &data_2),
            DualRangeAlgo::Beta => beta(&data_1, &data_2),
            DualRangeAlgo::WeightedMean => weighted_mean(&data_1, &data_2),
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

        // Return insight
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
