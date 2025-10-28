use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use serde::Deserialize;
use strum::Display;
use time::UtcDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{math::*, Feature, FeatureStore, FillStrategy, InstrumentScope};

#[derive(Debug, Display, Clone, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RangeData {
    Window(u64),
    Interval(usize),
}

#[derive(Debug, Display, Clone, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RangeAlgo {
    // Basic
    Count,
    Sum,
    SumPositive,
    SumNegative,
    AbsSum,
    SumAbsPositive,
    SumAbsNegative,
    Mean,
    Median,
    Min,
    Max,
    AbsolutRange,
    RelativeRange,
    RelativePosition,

    // Distribution
    Variance,
    StandardDeviation,
    AnnualizedVolatility,
    Skew,
    Kurtosis,
    Quantile(f64),
    Iqr,

    // Relationship
    Autocorrelation(usize),

    // Other
    CoefOfVariation,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct RangeFeature {
    input: FeatureId,
    output: FeatureId,
    method: RangeAlgo,
    data: RangeData,
    fill_strategy: FillStrategy,
    scopes: Vec<InstrumentScope>,
}

#[async_trait]
impl Feature for RangeFeature {
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
                // Collect input values from all input instruments in this scope
                let input_values: Vec<f64> = scope
                    .inputs
                    .iter()
                    .flat_map(|instrument| {
                        let values = match &self.data {
                            RangeData::Window(window_secs) => {
                                // Get values within time window
                                let window = Duration::from_secs(*window_secs);
                                state.window(instrument, &self.input, event_time, window)
                            }
                            RangeData::Interval(intervals) => {
                                // Get last N interval values
                                state
                                    .interval(instrument, &self.input, event_time, *intervals, Some(self.fill_strategy))
                                    .unwrap_or_default()
                            }
                        };

                        debug!(
                            "Feature {} reading {} from {}: got {} values",
                            self.output,
                            self.input,
                            instrument.symbol,
                            values.len()
                        );

                        values
                    })
                    .collect();

                // If no data available, apply fill strategy
                if input_values.is_empty() {
                    let value = match self.fill_strategy {
                        FillStrategy::Zero => 0.0,
                        FillStrategy::ForwardFill => {
                            // Try to get last value for output instrument
                            state.last(&scope.output, &self.output, event_time).unwrap_or(0.0)
                        }
                        FillStrategy::Drop => return None, // Skip this scope
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

                // Apply the range method to compute the output value
                let value = match &self.method {
                    RangeAlgo::Count => input_values.len() as f64,
                    RangeAlgo::Sum => input_values.iter().sum(),
                    RangeAlgo::SumPositive => input_values.iter().filter(|&&x| x > 0.0).sum(),
                    RangeAlgo::SumNegative => input_values.iter().filter(|&&x| x < 0.0).sum(),
                    RangeAlgo::AbsSum => input_values.iter().map(|x| x.abs()).sum(),
                    RangeAlgo::SumAbsPositive => input_values.iter().filter(|&&x| x > 0.0).map(|x| x.abs()).sum(),
                    RangeAlgo::SumAbsNegative => input_values.iter().filter(|&&x| x < 0.0).map(|x| x.abs()).sum(),
                    RangeAlgo::Mean => mean(&input_values),
                    RangeAlgo::Median => median(&input_values),
                    RangeAlgo::Min => min(&input_values),
                    RangeAlgo::Max => max(&input_values),
                    RangeAlgo::AbsolutRange => absolut_range(&input_values),
                    RangeAlgo::RelativeRange => relative_range(&input_values),
                    RangeAlgo::RelativePosition => relative_position(&input_values),
                    RangeAlgo::Variance => variance(&input_values),
                    RangeAlgo::StandardDeviation => std_dev(&input_values),
                    RangeAlgo::AnnualizedVolatility => annualized_volatility(&input_values),
                    RangeAlgo::Skew => skew(&input_values),
                    RangeAlgo::Kurtosis => kurtosis(&input_values),
                    RangeAlgo::Quantile(q) => quantile(&input_values, *q),
                    RangeAlgo::Iqr => iqr(&input_values),
                    RangeAlgo::Autocorrelation(lag) => autocorrelation(&input_values, *lag),
                    RangeAlgo::CoefOfVariation => coefficient_of_variation(&input_values),
                };

                debug!(
                    "Feature {} for {}: computed {} from {} input values",
                    self.output,
                    scope.output.symbol,
                    value,
                    input_values.len()
                );

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
