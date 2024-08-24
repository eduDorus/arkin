use arkin_common::prelude::*;
use rayon::prelude::*;
use time::OffsetDateTime;

use crate::{config::StrategyManagerConfig, factory::StrategyFactory};

pub trait StrategyModule: Send + Sync {
    fn id(&self) -> &StrategyId;
    fn sources(&self) -> &[FeatureId];
    fn calculate(&self, data: &[Feature]) -> Vec<Signal>;
}

pub struct StrategyManager {
    strategies: Vec<Box<dyn StrategyModule>>,
}

impl StrategyManager {
    pub fn from_config(config: &StrategyManagerConfig) -> Self {
        Self {
            strategies: StrategyFactory::from_config(&config.strategies),
        }
    }

    pub fn calculate(&self, timestamp: &OffsetDateTime, features: &FeatureSnapshot) -> SignalSnapshot {
        // Calculate signals for each strategy
        let signals = self
            .strategies
            .par_iter()
            .map(|s| s.calculate(&features.metrics))
            .flat_map(|s| s)
            .collect::<Vec<_>>();

        // Return signals
        SignalSnapshot::new(timestamp.to_owned(), signals)
    }
}
