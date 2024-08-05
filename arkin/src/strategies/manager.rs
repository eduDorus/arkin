use crate::{config::StrategyManagerConfig, features::FeatureEvent, models::Signal};
use rayon::prelude::*;

use super::{factory::StrategyFactory, Strategy};

pub struct StrategyManager {
    strategies: Vec<Box<dyn Strategy>>,
}

impl StrategyManager {
    pub fn from_config(config: &StrategyManagerConfig) -> Self {
        Self {
            strategies: StrategyFactory::from_config(&config.strategies),
        }
    }

    pub fn calculate(&self, data: Vec<FeatureEvent>) -> Vec<Signal> {
        self.strategies
            .par_iter()
            .map(|s| {
                // Filter data
                let data = data.iter().filter(|d| s.sources().contains(&d.id)).cloned().collect::<Vec<_>>();
                
                s.calculate(data)
            })
            .flat_map(|s| s)
            .collect::<Vec<_>>()
    }
}
