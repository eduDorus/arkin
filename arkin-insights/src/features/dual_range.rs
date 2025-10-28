use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use serde::Deserialize;
use strum::Display;
use time::UtcDateTime;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{features::RangeData, math::*, Feature, FeatureStore, FillStrategy, InstrumentScope};

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
    fill_strategy: FillStrategy,
    scopes: Vec<InstrumentScope>,
}

#[async_trait]
impl Feature for DualRangeFeature {
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
                // Collect values for both inputs from all input instruments
                let values_1: Vec<f64> = scope
                    .inputs
                    .iter()
                    .flat_map(|instrument| match &self.data {
                        RangeData::Window(window_secs) => {
                            let window = Duration::from_secs(*window_secs);
                            state.window(instrument, &self.input_1, event_time, window)
                        }
                        RangeData::Interval(intervals) => state
                            .interval(instrument, &self.input_1, event_time, *intervals, Some(self.fill_strategy))
                            .unwrap_or_default(),
                    })
                    .collect();

                let values_2: Vec<f64> = scope
                    .inputs
                    .iter()
                    .flat_map(|instrument| match &self.data {
                        RangeData::Window(window_secs) => {
                            let window = Duration::from_secs(*window_secs);
                            state.window(instrument, &self.input_2, event_time, window)
                        }
                        RangeData::Interval(intervals) => state
                            .interval(instrument, &self.input_2, event_time, *intervals, Some(self.fill_strategy))
                            .unwrap_or_default(),
                    })
                    .collect();

                // If no data available, apply fill strategy
                if values_1.is_empty() || values_2.is_empty() {
                    let value = match self.fill_strategy {
                        FillStrategy::Zero => 0.0,
                        FillStrategy::ForwardFill => state.last(&scope.output, &self.output, event_time).unwrap_or(0.0),
                        FillStrategy::Drop => return None,
                    };

                    return Some(Arc::new(
                        Insight::builder()
                            .event_time(event_time)
                            .pipeline(Some(pipeline.clone()))
                            .instrument(scope.output.clone())
                            .feature_id(self.output.clone())
                            .value(value)
                            .insight_type(InsightType::Continuous)
                            .build(),
                    ));
                }

                // Apply the dual range method
                let value = match &self.method {
                    DualRangeAlgo::Covariance => covariance(&values_1, &values_2),
                    DualRangeAlgo::Correlation => correlation(&values_1, &values_2),
                    DualRangeAlgo::CosineSimilarity => cosine_similarity(&values_1, &values_2),
                    DualRangeAlgo::Beta => beta(&values_1, &values_2),
                    DualRangeAlgo::WeightedMean => weighted_mean(&values_1, &values_2),
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
