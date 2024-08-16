mod crossover;
mod errors;
mod factory;
mod manager;

pub use manager::StrategyManager;

use crate::models::{FeatureEvent, FeatureId, Signal, StrategyId};

pub trait Strategy: Send + Sync {
    fn id(&self) -> &StrategyId;
    fn sources(&self) -> &[FeatureId];
    fn calculate(&self, data: &[FeatureEvent]) -> Vec<Signal>;
}
