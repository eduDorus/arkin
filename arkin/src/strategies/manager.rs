use super::{factory::StrategyFactory, Strategy};
use crate::{config::StrategyManagerConfig, features::FeatureEvent, models::Signal};
use rayon::prelude::*;

pub struct StrategyManager {
    strategies: Vec<Box<dyn Strategy>>,
}

impl StrategyManager {
    pub fn from_config(config: &StrategyManagerConfig) -> Self {
        Self {
            strategies: StrategyFactory::from_config(&config.strategies),
        }
    }

    pub fn calculate(&self, data: &[FeatureEvent]) -> Vec<Signal> {
        self.strategies
            .par_iter()
            .map(|s| s.calculate(data))
            .flat_map(|s| s)
            .collect::<Vec<_>>()
    }
}
