use arkin_common::prelude::*;
use rayon::prelude::*;

use crate::{config::StrategyManagerConfig, factory::StrategyFactory};

pub trait StrategyModule: Send + Sync {
    fn id(&self) -> &StrategyId;
    fn sources(&self) -> &[FeatureId];
    fn calculate(&self, data: &[Insight]) -> Vec<Signal>;
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

    pub fn calculate(&self, snapshot: &Snapshot) -> Vec<Signal> {
        // if snapshot.insights.is_empty() {
        //     return vec![];
        // }

        // // Calculate signals for each strategy
        // self.strategies
        //     .par_iter()
        //     .map(|s| s.calculate(&snapshot.insights))
        //     .flat_map(|s| s)
        //     .collect::<Vec<_>>()
        todo!()
    }
}
