use std::sync::Arc;

use serde::Deserialize;
use strum::Display;
use time::OffsetDateTime;
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
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    input_1: FeatureId,
    input_2: FeatureId,
    output: FeatureId,
    method: TwoValueAlgo,
    persist: bool,
}

impl Feature for TwoValueFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input_1.clone(), self.input_2.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, event_time: OffsetDateTime) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating {}...", self.method);

        //  Get data
        let value_1 = self
            .insight_state
            .last(Some(instrument.clone()), self.input_1.clone(), event_time);
        let value_2 = self
            .insight_state
            .last(Some(instrument.clone()), self.input_2.clone(), event_time);

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
        if self.method == TwoValueAlgo::Imbalance {
            if value_1 < 0.0 || value_2 < 0.0 {
                warn!("Imbalance values must be positive");
                return None;
            }
        }

        // Unwrap values

        let change = match self.method {
            TwoValueAlgo::Imbalance => imbalance(value_1, value_2),
            TwoValueAlgo::Elasticity => elasticity(value_1, value_2),
            TwoValueAlgo::Division => value_1 / value_2,
        };

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
