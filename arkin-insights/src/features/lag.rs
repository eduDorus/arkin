use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use strum::Display;
use time::UtcDateTime;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{Feature, FeatureStore, FillStrategy, InstrumentScope};

#[derive(Debug, Display, Clone, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LagAlgo {
    // Change
    AbsoluteChange,
    PercentChange,
    LogChange,
    Difference,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct LagFeature {
    input: FeatureId,
    output: FeatureId,
    lag: usize,
    method: LagAlgo,
    fill_strategy: FillStrategy,
    scopes: Vec<InstrumentScope>,
}

#[async_trait]
impl Feature for LagFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn fill_strategy(&self) -> FillStrategy {
        self.fill_strategy
    }

    fn scopes(&self) -> &[InstrumentScope] {
        &self.scopes
    }

    fn calculate(
        &self,
        state: &FeatureStore,
        pipeline: &Arc<Pipeline>,
        event_time: UtcDateTime,
    ) -> Option<Vec<Arc<Insight>>> {
        // Iterate over all scopes and compute for each
        let insights: Vec<Arc<Insight>> = self
            .scopes
            .iter()
            .filter_map(|scope| {
                // For lag features, we need current value and lagged value
                // Get current value from any of the input instruments
                let current_value = scope
                    .inputs
                    .iter()
                    .find_map(|instrument| state.last(instrument, &self.input, event_time))?;

                // Get lagged value
                let lagged_value = scope.inputs.iter().find_map(|instrument| {
                    state
                        .lag(instrument, &self.input, event_time, self.lag, Some(self.fill_strategy))
                        .ok()
                })?;

                // Apply the lag method to compute the change
                let value = match &self.method {
                    LagAlgo::AbsoluteChange => current_value - lagged_value,
                    LagAlgo::PercentChange => {
                        if lagged_value == 0.0 {
                            0.0
                        } else {
                            (current_value - lagged_value) / lagged_value.abs() * 100.0
                        }
                    }
                    LagAlgo::LogChange => {
                        if lagged_value <= 0.0 || current_value <= 0.0 {
                            0.0
                        } else {
                            (current_value / lagged_value).ln()
                        }
                    }
                    LagAlgo::Difference => current_value - lagged_value,
                };

                // Create insight for the output instrument
                Some(Arc::new(
                    Insight::builder()
                        .event_time(event_time)
                        .pipeline(Some(pipeline.clone()))
                        .instrument(scope.output.clone())
                        .feature_id(self.output.clone())
                        .value(value)
                        .insight_type(InsightType::Continuous)
                        .build(),
                ))
            })
            .collect();

        if insights.is_empty() {
            None
        } else {
            Some(insights)
        }
    }

    // async fn async_calculate(&self, instrument: &Arc<Instrument>, timestamp: UtcDateTime) -> Option<Vec<Insight>> {
    //     self.calculate(instrument, timestamp)
    // }
}
