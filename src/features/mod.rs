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
    fn calculate(&self);
    async fn calculate_async(&self);
}

#[derive(Debug)]
pub struct BaseGen {
    id: FeatureID,
}

impl Default for BaseGen {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseGen {
    pub fn new() -> Self {
        BaseGen {
            id: FeatureID::from("base"),
        }
    }
}

#[async_trait]
impl Feature for BaseGen {
    fn id(&self) -> &FeatureID {
        &self.id
    }

    fn sources(&self) -> Vec<&FeatureID> {
        vec![]
    }

    fn calculate(&self) {
        info!("Calculating Base with id: {}", self.id);
    }

    async fn calculate_async(&self) {
        info!("Calculating Base with id: {}", self.id);
    }
}

#[derive(Debug)]
pub struct VolumeGen {
    id: FeatureID,
    window: Duration,
}

impl VolumeGen {
    pub fn new(id: FeatureID, window: Duration) -> Self {
        VolumeGen { id, window }
    }
}

#[async_trait]
impl Feature for VolumeGen {
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
        info!("Calculating Volume with id: {}", self.id);

        // Generate a random limit for the Fibonacci calculation
        let limit = rand::thread_rng().gen_range(40..45); // Adjust the range as needed

        // Perform the Fibonacci computation
        let result = fibonacci(limit);
        info!("Volume result for {}: {}", limit, result);
    }
}

#[derive(Debug)]
pub struct VWAPGen {
    id: FeatureID,
    window: Duration,
}

impl VWAPGen {
    pub fn new(id: FeatureID, window: Duration) -> Self {
        VWAPGen { id, window }
    }
}

#[async_trait]
impl Feature for VWAPGen {
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
        info!("Calculating VWAP with id: {}", self.id);
        // Generate a random limit for the Fibonacci calculation
        let limit = rand::thread_rng().gen_range(40..45); // Adjust the range as needed

        // Perform the Fibonacci computation
        let result = fibonacci(limit);
        info!("VWAP result for {}: {}", limit, result);
    }
}

#[derive(Debug)]
pub struct SMAGen {
    id: FeatureID,
    source: FeatureID,
    window: Duration,
}

impl SMAGen {
    pub fn new(id: FeatureID, source: FeatureID, window: Duration) -> Self {
        SMAGen { id, source, window }
    }
}

#[async_trait]
impl Feature for SMAGen {
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
        info!("Calculating SMA with id: {}", self.id);
        // Wait a random amount of time between 0 and 1 second
        // Generate a random limit for the Fibonacci calculation
        let limit = rand::thread_rng().gen_range(40..45); // Adjust the range as needed

        // Perform the Fibonacci computation
        let result = fibonacci(limit);
        info!("SMA result for {}: {}", limit, result);
    }
}

#[derive(Debug)]
pub struct EMAGen {
    id: FeatureID,
    source: FeatureID,
    window: Duration,
}

impl EMAGen {
    pub fn new(id: FeatureID, source: FeatureID, window: Duration) -> Self {
        EMAGen { id, source, window }
    }
}

#[async_trait]
impl Feature for EMAGen {
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
        info!("Calculating EMA with id: {}", self.id);
        // Wait a random amount of time between 0 and 1 second
        // Generate a random limit for the Fibonacci calculation
        let limit = rand::thread_rng().gen_range(40..45); // Adjust the range as needed

        // Perform the Fibonacci computation
        let result = fibonacci(limit);
        info!("EMA result for {}: {}", limit, result);
    }
}

#[derive(Debug)]
pub struct SpreadGen {
    id: FeatureID,
    front_component: FeatureID,
    back_component: FeatureID,
}

impl SpreadGen {
    pub fn new(id: FeatureID, front_component: FeatureID, back_component: FeatureID) -> Self {
        SpreadGen {
            id,
            front_component,
            back_component,
        }
    }
}

#[async_trait]
impl Feature for SpreadGen {
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
        info!("Calculating Spread with id: {}", self.id);
        // Wait a random amount of time between 0 and 1 second
        // Generate a random limit for the Fibonacci calculation
        let limit = rand::thread_rng().gen_range(40..45); // Adjust the range as needed

        // Perform the Fibonacci computation
        let result = fibonacci(limit);
        info!("Spread result for {}: {}", limit, result);
    }
}
