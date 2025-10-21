use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use strum::Display;
use time::UtcDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{math::*, state::InsightsState, Feature};

#[derive(Debug, Display, Clone, Deserialize, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TwoValueAlgo {
    Imbalance,
    Elasticity,
    Division,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct TwoValueFeature {
    input_1: FeatureId,
    input_2: FeatureId,
    output: FeatureId,
    method: TwoValueAlgo,
    persist: bool,
}

#[async_trait]
impl Feature for TwoValueFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input_1.clone(), self.input_2.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(
        &self,
        state: &InsightsState,
        pipeline: &Arc<Pipeline>,
        instrument: &Arc<Instrument>,
        event_time: UtcDateTime,
    ) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating {}...", self.method);

        //  Get data
        let value_1 = state.last(instrument, &self.input_1, event_time);
        let value_2 = state.last(instrument, &self.input_2, event_time);

        // Check if we have enough data
        if value_1.is_none() || value_2.is_none() {
            warn!(
                "Not enough data for {} value {:?}, lag value {:?}",
                self.output, value_1, value_2
            );
            return None;
        }

        let value_1 = value_1.expect("Value 1 should not be None");
        let value_2 = value_2.expect("Value 2 should not be None");

        // If our method is imbalance we need to make sure the values are positve
        if self.method == TwoValueAlgo::Imbalance && (value_1 < 0.0 || value_2 < 0.0) {
            warn!("Imbalance values must be positive");
            return None;
        }

        let mut change = match self.method {
            TwoValueAlgo::Imbalance => imbalance(value_1, value_2),
            TwoValueAlgo::Elasticity => elasticity(value_1, value_2),
            TwoValueAlgo::Division => value_1 / value_2,
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
