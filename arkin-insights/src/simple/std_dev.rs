use std::sync::Arc;

use rust_decimal::prelude::*;
use time::OffsetDateTime;
use tracing::{debug, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{state::InsightsState, Feature};

#[derive(Debug, Clone, TypedBuilder)]
pub struct StdDevFeature {
    pipeline: Arc<Pipeline>,
    insight_state: Arc<InsightsState>,
    input: FeatureId,
    output: FeatureId,
    periods: usize,
    persist: bool,
}

impl Feature for StdDevFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: &Arc<Instrument>, event_time: OffsetDateTime) -> Option<Vec<Arc<Insight>>> {
        debug!("Calculating StdDev...");

        // Get data from state
        let data = self
            .insight_state
            .periods(Some(instrument.clone()), self.input.clone(), event_time, self.periods);

        // Check if we have enough data
        if data.len() < self.periods {
            warn!("Not enough data for StdDev calculation");
            return None;
        }

        // Calculate StdDev
        let sum = data.iter().sum::<f64>();
        let count = data.len() as f64;
        let mean = match count.is_zero() {
            true => {
                warn!("Count should not be zero!");
                return None;
            }
            false => sum / count,
        };
        let variance = (1. / (count - 1.)) * data.iter().map(|v| (v - mean).powi(2)).sum::<f64>();
        let std_dev = variance.sqrt();

        let insight = Insight::builder()
            .event_time(event_time)
            .pipeline(Some(self.pipeline.clone()))
            .instrument(Some(instrument.clone()))
            .feature_id(self.output.clone())
            .value(std_dev)
            .insight_type(InsightType::Continuous)
            .persist(self.persist)
            .build()
            .into();

        Some(vec![insight])
    }
}
