use serde::{Deserialize, Serialize};
use std::fmt::{self, Debug};

mod crossover;
mod errors;
mod factory;
mod manager;

pub use manager::StrategyManager;

use crate::{
    features::{FeatureEvent, FeatureId},
    models::Signal,
};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct StrategyId(String);

impl From<&str> for StrategyId {
    fn from(id: &str) -> Self {
        StrategyId(id.to_lowercase())
    }
}

impl From<String> for StrategyId {
    fn from(id: String) -> Self {
        StrategyId(id.to_lowercase())
    }
}

impl fmt::Display for StrategyId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait Strategy: Debug + Send + Sync {
    fn id(&self) -> &StrategyId;
    fn sources(&self) -> Vec<FeatureId>;
    fn calculate(&self, data: Vec<FeatureEvent>) -> Vec<Signal>;
}
