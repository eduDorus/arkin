use std::sync::Arc;

use time::OffsetDateTime;
use tracing::warn;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{
    config::DataLoaderConfig, state::InsightsState, Feature, MultiVecComputation, SingleVecComputation,
    TwoValueComputation, TwoVecComputation,
};

pub trait Feature: Send + Sync {
    fn inputs(&self) -> Vec<FeatureId>;
    fn outputs(&self) -> Vec<FeatureId>;
    fn calculate(&self, instrument: Arc<Instrument>, event_time: OffsetDateTime) -> Vec<Arc<Insight>>;
}

pub trait GlobalFeature: Send + Sync {
    fn inputs(&self) -> Vec<FeatureId>;
    fn outputs(&self) -> Vec<FeatureId>;
    fn calculate(&self, instruments: &[Arc<Instrument>], event_time: OffsetDateTime) -> Vec<Arc<Insight>>;
}

pub trait TwoValueComputation: Send + Sync {
    fn compute(&self, value1: f64, value2: f64) -> Option<f64>;
}

// Single-vector computations (e.g., sum, mean)
pub trait SingleVecComputation: Send + Sync {
    fn compute(&self, data: &[f64]) -> Option<f64>;
}

// Two-vector computations (e.g., covariance, correlation)
pub trait TwoVecComputation: Send + Sync {
    fn compute(&self, data1: &[f64], data2: &[f64]) -> Option<f64>;
}

// Matrix computations (e.g., PCA)
pub trait MultiVecComputation: Send + Sync {
    fn compute(&self, data: HashMap<FeatureId, Vec<f64>>) -> Option<f64>;
}

#[derive(Clone, TypedBuilder)]
pub struct TwoValueFeature<T: TwoValueComputation> {
    state: Arc<InsightsState>,
    pipeline: Arc<Pipeline>,
    input1: FeatureId,
    input1_lag: DataLoaderConfig,
    input2: FeatureId,
    input2_lag: DataLoaderConfig,
    output: FeatureId,
    computation: T,
}

impl<T: TwoValueComputation> Feature for TwoValueFeature<T> {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input1.clone(), self.input2.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: Arc<Instrument>, event_time: OffsetDateTime) -> Vec<Arc<Insight>> {
        // Load data
        let lag_1 = match self.input1_lag {
            DataLoaderConfig::Lag(lag) => lag,
            _ => {
                warn!("Unsupported data loader for TwoValueFeature");
                return vec![];
            }
        };
        let data1 = self
            .state
            .lag(Some(instrument.clone()), self.input1.clone(), event_time, lag_1)
            .map(|v| vec![v])
            .unwrap_or_default();

        let lag_2 = match self.input2_lag {
            DataLoaderConfig::Lag(lag) => lag,
            _ => {
                warn!("Unsupported data loader for TwoValueFeature");
                return vec![];
            }
        };
        let data2 = self
            .state
            .lag(Some(instrument.clone()), self.input2.clone(), event_time, lag_2)
            .map(|v| vec![v])
            .unwrap_or_default();

        // Unpack the values from the data
        if data1.is_empty() || data2.is_empty() {
            warn!("No data for Feature");
            return vec![];
        }

        if data1.len() > 1 || data2.len() > 1 {
            warn!("Too much data for Feature");
            return vec![];
        }
        let data1 = data1[0];
        let data2 = data2[0];

        // Compute
        if let Some(value) = self.computation.compute(data1, data2) {
            let insight = Insight::builder()
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output.clone())
                .event_time(event_time)
                .value(value)
                .persist(true)
                .build()
                .into();
            vec![insight]
        } else {
            vec![]
        }
    }
}

#[derive(Clone, TypedBuilder)]
pub struct SingleVecFeature<T: SingleVecComputation> {
    state: Arc<InsightsState>,
    pipeline: Arc<Pipeline>,
    input: FeatureId,
    output: FeatureId,
    data_loader: DataLoaderConfig,
    computation: T,
}

impl<T: SingleVecComputation> Feature for SingleVecFeature<T> {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: Arc<Instrument>, event_time: OffsetDateTime) -> Vec<Arc<Insight>> {
        // Load data
        let data = match self.data_loader {
            DataLoaderConfig::Periods(periods) => {
                self.state
                    .periods(Some(instrument.clone()), self.input.clone(), event_time, periods)
            }
            DataLoaderConfig::Window(window) => {
                self.state
                    .window(Some(instrument.clone()), self.input.clone(), event_time, window)
            }
            DataLoaderConfig::Lag(_) => {
                warn!("Unsupported data loader for SingleVecFeature");
                return vec![];
            }
        };

        // Compute
        if let Some(value) = self.computation.compute(&data) {
            let insight = Insight::builder()
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output.clone())
                .event_time(event_time)
                .value(value)
                .persist(true)
                .build()
                .into();
            vec![insight]
        } else {
            vec![]
        }
    }
}

#[derive(Clone)]
pub struct TwoVecFeature<T: TwoVecComputation> {
    state: Arc<InsightsState>,
    pipeline: Arc<Pipeline>,
    input1: FeatureId,
    input2: FeatureId,
    output: FeatureId,
    data_loader: DataLoaderConfig,
    computation: T,
}

impl<T: TwoVecComputation> Feature for TwoVecFeature<T> {
    fn inputs(&self) -> Vec<FeatureId> {
        vec![self.input1.clone(), self.input2.clone()]
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: Arc<Instrument>, event_time: OffsetDateTime) -> Vec<Arc<Insight>> {
        // Load data
        let (data1, data2) = match self.data_loader {
            DataLoaderConfig::Periods(periods) => {
                let data1 = self
                    .state
                    .periods(Some(instrument.clone()), self.input1.clone(), event_time, periods);
                let data2 = self
                    .state
                    .periods(Some(instrument.clone()), self.input2.clone(), event_time, periods);
                (data1, data2)
            }
            DataLoaderConfig::Window(window) => {
                let data1 = self
                    .state
                    .window(Some(instrument.clone()), self.input1.clone(), event_time, window);
                let data2 = self
                    .state
                    .window(Some(instrument.clone()), self.input2.clone(), event_time, window);
                (data1, data2)
            }
            DataLoaderConfig::Lag(_) => {
                warn!("Unsupported data loader for TwoVecFeature");
                return vec![];
            }
        };

        // Compute
        if let Some(value) = self.computation.compute(&data1, &data2) {
            let insight = Insight::builder()
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output.clone())
                .event_time(event_time)
                .value(value)
                .persist(true)
                .build()
                .into();
            vec![insight]
        } else {
            vec![]
        }
    }
}

#[derive(Clone)]
pub struct MultiVecFeature<T: MultiVecComputation> {
    state: Arc<InsightsState>,
    pipeline: Arc<Pipeline>,
    inputs: Vec<FeatureId>,
    output: FeatureId,
    data_loader: DataLoaderConfig,
    computation: T,
}

impl<T: MultiVecComputation> Feature for MultiVecFeature<T> {
    fn inputs(&self) -> Vec<FeatureId> {
        self.inputs.clone()
    }

    fn outputs(&self) -> Vec<FeatureId> {
        vec![self.output.clone()]
    }

    fn calculate(&self, instrument: Arc<Instrument>, event_time: OffsetDateTime) -> Vec<Arc<Insight>> {
        // Load data
        let data = match self.data_loader {
            DataLoaderConfig::Periods(periods) => self
                .inputs
                .iter()
                .map(|input| {
                    (
                        input.clone(),
                        self.state.periods(Some(instrument.clone()), input.clone(), event_time, periods),
                    )
                })
                .collect(),
            DataLoaderConfig::Window(window) => self
                .inputs
                .iter()
                .map(|input| {
                    (
                        input.clone(),
                        self.state.window(Some(instrument.clone()), input.clone(), event_time, window),
                    )
                })
                .collect(),
            DataLoaderConfig::Lag(_) => {
                warn!("Unsupported data loader for MultiVecFeature");
                return vec![];
            }
        };

        // Compute
        if let Some(value) = self.computation.compute(data) {
            let insight = Insight::builder()
                .pipeline(Some(self.pipeline.clone()))
                .instrument(Some(instrument.clone()))
                .feature_id(self.output.clone())
                .event_time(event_time)
                .value(value)
                .persist(true)
                .build()
                .into();
            vec![insight]
        } else {
            vec![]
        }
    }
}
