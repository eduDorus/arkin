use arkin_common::prelude::*;
use rayon::prelude::*;

use crate::{config::StrategyManagerConfig, factory::StrategyFactory};

pub trait StrategyModule: Send + Sync {
    fn id(&self) -> &StrategyId;
    fn sources(&self) -> &[FeatureId];
    fn calculate(&self, insights: &InsightsSnapshot) -> Vec<Signal>;
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

    pub fn process(&self, insights: &InsightsSnapshot) -> StrategySnapshot {
        if insights.insights.is_empty() {
            return {
                let timestamp = insights.timestamp;
                let signals = Vec::new();
                StrategySnapshot { timestamp, signals }
            };
        }

        // Calculate signals for each strategy
        let signals = self
            .strategies
            .par_iter()
            .map(|s| s.calculate(insights))
            .flat_map(|s| s)
            .collect::<Vec<_>>();

        {
            let timestamp = insights.timestamp;
            StrategySnapshot { timestamp, signals }
        }
    }
}
