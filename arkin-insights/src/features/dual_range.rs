use std::{sync::Arc, time::Duration};

use serde::Deserialize;
use strum::Display;
use time::OffsetDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{features::RangeData, math::*, state::InsightsState, Feature};

#[derive(Debug, Display, Clone, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DualRangeAlgo {
    Covariance,
    Correlation,
    CosineSimilarity,
    Beta,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct DualRangeFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    input_1: FeatureId,
    input_2: FeatureId,
    output: FeatureId,
    method: DualRangeAlgo,
    data: RangeData,
    persist: bool,
}

impl Feature for DualRangeFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input_1.clone(), self.input_2.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, event_time: OffsetDateTime) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating {}...", self.method);

        // Get data
        let data_1 = match self.data {
            RangeData::Interval(i) => {
                self.insight_state
                    .intervals(Some(instrument.clone()), self.input_1.clone(), event_time, i)
            }
            RangeData::Window(w) => self.insight_state.window(
                Some(instrument.clone()),
                self.input_1.clone(),
                event_time,
                Duration::from_secs(w),
            ),
        };
        let data_2 = match self.data {
            RangeData::Interval(i) => {
                self.insight_state
                    .intervals(Some(instrument.clone()), self.input_2.clone(), event_time, i)
            }
            RangeData::Window(w) => self.insight_state.window(
                Some(instrument.clone()),
                self.input_2.clone(),
                event_time,
                Duration::from_secs(w),
            ),
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
        let value = match self.method {
            DualRangeAlgo::Covariance => covariance(&data_1, &data_2),
            DualRangeAlgo::Correlation => correlation(&data_1, &data_2),
            DualRangeAlgo::CosineSimilarity => cosine_similarity(&data_1, &data_2),
            DualRangeAlgo::Beta => beta(&data_1, &data_2),
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
