use std::collections::HashMap;

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

    pub fn calculate(&self, snapshot: &InsightsSnapshot) -> StrategySnapshot {
        if snapshot.insights.is_empty() {
            return StrategySnapshot::new(snapshot.event_time, HashMap::new());
        }

        // Calculate signals for each strategy
        let signals = self
            .strategies
            .par_iter()
            .map(|s| s.calculate(&snapshot))
            .flat_map(|s| s)
            .map(|s| ((s.strategy_id.clone(), s.instrument.clone()), s))
            .collect::<HashMap<_, _>>();

        StrategySnapshot::new(snapshot.event_time, signals)
    }
}
