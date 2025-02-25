use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use arkin_core::prelude::*;
use typed_builder::TypedBuilder;

use crate::{config::DataLoaderConfig, state::InsightsState};

pub trait Computation: std::fmt::Debug + Send + Sync {
    fn inputs(&self) -> Vec<FeatureId>;
    fn outputs(&self) -> Vec<FeatureId>;
    fn calculate(&self, instruments: &[Arc<Instrument>], event_time: OffsetDateTime) -> Result<Vec<Arc<Insight>>>;
}

#[derive(Serialize, Deserialize)]
pub struct NewPipelineConfig {
    pub features: Vec<NewFeatureConfig>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NewFeatureConfig {
    StdDev(SingleVecFeatureConfig),
    Ratio(TwoValueFeatureConfig),
}

#[derive(Serialize, Deserialize)]
pub struct SingleVecFeatureConfig {
    pub input: FeatureId,
    pub output: FeatureId,
    pub data_loader: DataLoaderConfig,
}

#[derive(Serialize, Deserialize)]
pub struct TwoValueFeatureConfig {
    pub input1: FeatureId,
    pub input2: FeatureId,
    pub output: FeatureId,
    pub data_loader: DataLoaderConfig,
}

pub struct NewPipeline {
    features: Vec<Arc<dyn Feature>>,
}

impl NewPipeline {
    pub fn from_config(state: Arc<InsightsState>, config: NewPipelineConfig) -> Self {
        let mut features: Vec<Arc<dyn Feature>> = vec![];
        for c in config.features {
            let feature = match c {
                NewFeatureConfig::StdDev(c) => SingleVecFeature::builder()
                    .input(c.input)
                    .output(c.output)
                    .data_loader(c.data_loader)
                    .computation(StdDevCalculation::default())
                    .pipeline(test_pipeline())
                    .state(state.clone())
                    .build(),
                _ => unimplemented!(),
            };
            features.push(Arc::new(feature));
        }
        Self { features }
    }

    pub fn calculate(&self, instrument: Arc<Instrument>, event_time: OffsetDateTime) -> Vec<Arc<Insight>> {
        self.features
            .iter()
            .flat_map(|feature| feature.calculate(instrument.clone(), event_time))
            .collect()
    }
}

#[derive(Default)]
pub struct StdDevCalculation {}

impl SingleVecComputation for StdDevCalculation {
    fn compute(&self, data: &[f64]) -> Option<f64> {
        Some(std_dev_f64_par(&data))
    }
}

fn sum_f64_par(data: &[f64]) -> f64 {
    data.par_iter().sum()
}

fn mean_f64_par(data: &[f64]) -> f64 {
    // if data.is_empty() {
    //     return None;
    // }
    let sum: f64 = sum_f64_par(data);
    let n = data.len() as f64;
    sum / n
}

fn variance_f64_par(data: &[f64]) -> f64 {
    // if data.len() < 2 {
    //     return None;
    // }
    let mean = mean_f64_par(data);
    let sum_sq_diff: f64 = data.par_iter().map(|x| (x - mean).powi(2)).sum();
    let n = data.len() as f64 - 1.0;
    sum_sq_diff / n
}

fn std_dev_f64_par(data: &[f64]) -> f64 {
    variance_f64_par(data).sqrt()
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
            DataLoaderConfig::LastValue => self
                .state
                .last(Some(instrument.clone()), self.input.clone(), event_time)
                .map(|v| vec![v])
                .unwrap_or_default(),
        };

        // Compute
        if let Some(value) = self.computation.compute(&data) {
            let insight = Insight::builder()
                .pipeline(self.pipeline.clone())
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
            DataLoaderConfig::LastValue => {
                let data1 = self
                    .state
                    .last(Some(instrument.clone()), self.input1.clone(), event_time)
                    .map(|v| vec![v])
                    .unwrap_or_default();
                let data2 = self
                    .state
                    .last(Some(instrument.clone()), self.input2.clone(), event_time)
                    .map(|v| vec![v])
                    .unwrap_or_default();
                (data1, data2)
            }
        };

        // Compute
        if let Some(value) = self.computation.compute(&data1, &data2) {
            let insight = Insight::builder()
                .pipeline(self.pipeline.clone())
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
            DataLoaderConfig::LastValue => self
                .inputs
                .iter()
                .map(|input| {
                    (
                        input.clone(),
                        self.state
                            .last(Some(instrument.clone()), input.clone(), event_time)
                            .map(|v| vec![v])
                            .unwrap_or_default(),
                    )
                })
                .collect(),
        };

        // Compute
        if let Some(value) = self.computation.compute(data) {
            let insight = Insight::builder()
                .pipeline(self.pipeline.clone())
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

pub trait Feature: Send + Sync {
    fn inputs(&self) -> Vec<FeatureId>;
    fn outputs(&self) -> Vec<FeatureId>;
    fn calculate(&self, instrument: Arc<Instrument>, event_time: OffsetDateTime) -> Vec<Arc<Insight>>;
}

pub trait TwoValueComputations: Send + Sync {
    fn compute(&self, value1: f64, value2: f64) -> f64;
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
