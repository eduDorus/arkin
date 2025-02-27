use std::sync::Arc;

use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{state::InsightsState, Feature};

#[derive(Debug, Clone, TypedBuilder)]
pub struct LogChange {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    input: FeatureId,
    output: FeatureId,
    periods: usize,
    persist: bool,
}

impl Feature for LogChange {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, event_time: OffsetDateTime) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating Log Returns...");

        //  Get data
        let data =
            self.insight_state
                .periods(Some(instrument.clone()), self.input.clone(), event_time, self.periods + 1);

        // Check if we have enough data
        if data.len() < self.periods + 1 {
            warn!("Not enough data to calculate log return");
            return None;
        }

        // Get values change
        let prev_value = data
            .first()
            .expect("Could not get first value, unexpected empty vector, should have been caught earlier");
        let last_value = data
            .last()
            .expect("Could not get last value, unexpected empty vector, should have been caught earlier");

        let log_return = if prev_value.is_zero() {
            return None;
        } else {
            (last_value / prev_value).ln()
        };

        // Return insight
        let insight = Insight::builder()
            .event_time(event_time)
            .pipeline(Some(self.pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(self.output.clone())
            .value(log_return)
            .persist(self.persist)
            .insight_type(InsightType::Continuous)
            .build()
            .into();

        Some(vec![insight])
    }
}
