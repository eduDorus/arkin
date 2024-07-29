pub mod errors;
mod factory;
mod graph;
// mod vwap;

use async_trait::async_trait;
use core::fmt;
pub use graph::Pipeline;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, time::Duration};
use tracing::info;

use crate::config::{EMAFeatureConfig, SMAFeatureConfig, SpreadFeatureConfig, VWAPFeatureConfig, VolumeFeatureConfig};

fn fibonacci(n: u64) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct FeatureID(String);

impl From<&str> for FeatureID {
    fn from(id: &str) -> Self {
        FeatureID(id.to_lowercase())
    }
}

impl From<String> for FeatureID {
    fn from(id: String) -> Self {
        FeatureID(id.to_lowercase())
    }
}

impl fmt::Display for FeatureID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[async_trait]
pub trait Feature: Debug + Send + Sync {
    fn id(&self) -> &FeatureID;
    fn sources(&self) -> Vec<&FeatureID>;
    // fn data(&self, store: Arc<DataStore>) -> QueryResult;
    fn calculate(&self);
    async fn calculate_async(&self);
}

#[derive(Debug)]
pub struct VolumeFeature {
    id: FeatureID,
    _window: Duration,
}

impl VolumeFeature {
    pub fn new(id: FeatureID, window: Duration) -> Self {
        VolumeFeature {
            id,
            _window: window,
        }
    }

    pub fn from_config(config: &VolumeFeatureConfig) -> Self {
        VolumeFeature {
            id: config.id.to_owned(),
            _window: Duration::from_secs(config.window),
        }
    }
}

#[async_trait]
impl Feature for VolumeFeature {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<&FeatureID> {
        vec![]
    }

    fn calculate(&self) {
        info!("Calculating Volume with id: {}", self.id);

        // Generate a random limit for the Fibonacci calculation
        let limit = rand::thread_rng().gen_range(40..45); // Adjust the range as needed

        // Perform the Fibonacci computation
        let result = fibonacci(limit);
        info!("Volume result for {}: {}", limit, result);
    }

    async fn calculate_async(&self) {
        self.calculate();
    }
}

#[derive(Debug)]
pub struct VWAPFeature {
    id: FeatureID,
    _window: Duration,
}

impl VWAPFeature {
    pub fn new(id: FeatureID, window: Duration) -> Self {
        VWAPFeature {
            id,
            _window: window,
        }
    }

    pub fn from_config(config: &VWAPFeatureConfig) -> Self {
        VWAPFeature {
            id: config.id.to_owned(),
            _window: Duration::from_secs(config.window),
        }
    }
}

#[async_trait]
impl Feature for VWAPFeature {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<&FeatureID> {
        vec![]
    }

    fn calculate(&self) {
        info!("Calculating VWAP with id: {}", self.id);
        // Generate a random limit for the Fibonacci calculation
        let limit = rand::thread_rng().gen_range(40..45); // Adjust the range as needed

        // Perform the Fibonacci computation
        let result = fibonacci(limit);
        info!("VWAP result for {}: {}", limit, result);
    }

    async fn calculate_async(&self) {
        self.calculate();
    }
}

#[derive(Debug)]
pub struct SMAFeature {
    id: FeatureID,
    source: FeatureID,
    _period: u64,
}

impl SMAFeature {
    pub fn new(id: FeatureID, source: FeatureID, period: u64) -> Self {
        SMAFeature {
            id,
            source,
            _period: period,
        }
    }

    pub fn from_config(config: &SMAFeatureConfig) -> Self {
        SMAFeature {
            id: config.id.to_owned(),
            source: config.source.to_owned(),
            _period: config.period,
        }
    }
}

#[async_trait]
impl Feature for SMAFeature {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<&FeatureID> {
        vec![&self.source]
    }

    fn calculate(&self) {
        info!("Calculating SMA with id: {}", self.id);
        // Wait a random amount of time between 0 and 1 second
        // Generate a random limit for the Fibonacci calculation
        let limit = rand::thread_rng().gen_range(40..45); // Adjust the range as needed

        // Perform the Fibonacci computation
        let result = fibonacci(limit);
        info!("SMA result for {}: {}", limit, result);
    }

    async fn calculate_async(&self) {
        self.calculate();
    }
}

#[derive(Debug)]
pub struct EMAFeature {
    id: FeatureID,
    source: FeatureID,
    _period: u64,
}

impl EMAFeature {
    pub fn new(id: FeatureID, source: FeatureID, period: u64) -> Self {
        EMAFeature {
            id,
            source,
            _period: period,
        }
    }

    pub fn from_config(config: &EMAFeatureConfig) -> Self {
        EMAFeature {
            id: config.id.to_owned(),
            source: config.source.to_owned(),
            _period: config.period,
        }
    }
}

#[async_trait]
impl Feature for EMAFeature {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<&FeatureID> {
        vec![&self.source]
    }

    fn calculate(&self) {
        info!("Calculating EMA with id: {}", self.id);
        // Wait a random amount of time between 0 and 1 second
        // Generate a random limit for the Fibonacci calculation
        let limit = rand::thread_rng().gen_range(40..45); // Adjust the range as needed

        // Perform the Fibonacci computation
        let result = fibonacci(limit);
        info!("EMA result for {}: {}", limit, result);
    }

    async fn calculate_async(&self) {
        self.calculate();
    }
}

#[derive(Debug)]
pub struct SpreadFeature {
    id: FeatureID,
    front_component: FeatureID,
    back_component: FeatureID,
}

impl SpreadFeature {
    pub fn new(id: FeatureID, front_component: FeatureID, back_component: FeatureID) -> Self {
        SpreadFeature {
            id,
            front_component,
            back_component,
        }
    }

    pub fn from_config(config: &SpreadFeatureConfig) -> Self {
        SpreadFeature {
            id: config.id.to_owned(),
            front_component: config.front_component.to_owned(),
            back_component: config.back_component.to_owned(),
        }
    }
}

#[async_trait]
impl Feature for SpreadFeature {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<&FeatureID> {
        vec![&self.front_component, &self.back_component]
    }

    fn calculate(&self) {
        info!("Calculating Spread with id: {}", self.id);
        // Wait a random amount of time between 0 and 1 second
        // Generate a random limit for the Fibonacci calculation
        let limit = rand::thread_rng().gen_range(40..45); // Adjust the range as needed

        // Perform the Fibonacci computation
        let result = fibonacci(limit);
        info!("Spread result for {}: {}", limit, result);
    }

    async fn calculate_async(&self) {
        self.calculate();
    }
}
