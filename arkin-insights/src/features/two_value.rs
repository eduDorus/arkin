use std::sync::Arc;

use async_trait::async_trait;
use serde::Deserialize;
use strum::Display;
use time::UtcDateTime;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{Feature, FeatureStore, FillStrategy, InstrumentScope};

#[derive(Debug, Display, Clone, Copy, Deserialize, PartialEq, Eq)]
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
    fill_strategy: FillStrategy,
    scopes: Vec<InstrumentScope>,
}

#[async_trait]
impl Feature for TwoValueFeature {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input_1.clone(), self.input_2.clone()]
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
                // For two-value features, we typically read the last value from each input
                // Get the last value for input_1 from all input instruments (should be same for all)
                let value_1 = scope
                    .inputs
                    .iter()
                    .find_map(|instrument| state.last(instrument, &self.input_1, event_time))?;

                // Get the last value for input_2
                let value_2 = scope
                    .inputs
                    .iter()
                    .find_map(|instrument| state.last(instrument, &self.input_2, event_time))?;

                // Apply the two-value method
                let value = match &self.method {
                    TwoValueAlgo::Imbalance => {
                        // Imbalance = (buy - sell) / (buy + sell)
                        if value_1 + value_2 == 0.0 {
                            0.0
                        } else {
                            (value_1 - value_2) / (value_1 + value_2)
                        }
                    }
                    TwoValueAlgo::Elasticity => {
                        // Elasticity = value_1 / value_2
                        if value_2 == 0.0 {
                            0.0
                        } else {
                            value_1 / value_2
                        }
                    }
                    TwoValueAlgo::Division => {
                        // Simple division
                        if value_2 == 0.0 {
                            0.0
                        } else {
                            value_1 / value_2
                        }
                    }
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
