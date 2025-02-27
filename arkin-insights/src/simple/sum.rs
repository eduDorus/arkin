use std::sync::Arc;

use time::OffsetDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{state::InsightsState, Feature};

#[derive(Debug, Clone, TypedBuilder)]
pub struct SumFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    input: FeatureId,
    output: FeatureId,
    periods: usize,
    persist: bool,
}

impl Feature for SumFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, event_time: OffsetDateTime) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating Sum...");

        // Get data from state
        let data = self
            .insight_state
            .periods(Some(instrument.clone()), self.input.clone(), event_time, self.periods);

        // Check if we have enough data
        if data.len() < self.periods {
            warn!("Not enough data for Sum calculation");
            return None;
        }

        // Calculate StdDev
        let sum = data.iter().sum::<f64>();

        let insight = Insight::builder()
            .event_time(event_time)
            .pipeline(Some(self.pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(self.output.clone())
            .value(sum)
            .insight_type(InsightType::Continuous)
            .persist(self.persist)
            .build()
            .into();

        Some(vec![insight])
    }
}
