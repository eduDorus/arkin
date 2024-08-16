use super::{factory::StrategyFactory, Strategy};
use crate::{
    config::StrategyManagerConfig,
    models::{FeatureSnapshot, SignalSnapshot},
    state::StateManager,
};
use rayon::prelude::*;
use std::sync::Arc;
use time::OffsetDateTime;

pub struct StrategyManager {
    state: Arc<StateManager>,
    strategies: Vec<Box<dyn Strategy>>,
}

impl StrategyManager {
    pub fn from_config(state: Arc<StateManager>, config: &StrategyManagerConfig) -> Self {
        Self {
            state,
            strategies: StrategyFactory::from_config(&config.strategies),
        }
    }

    pub fn calculate(&self, timestamp: &OffsetDateTime, features: &FeatureSnapshot) -> SignalSnapshot {
        // Calculate signals for each strategy
        let signals = self
            .strategies
            .par_iter()
            .map(|s| s.calculate(&features.features))
            .flat_map(|s| s)
            .collect::<Vec<_>>();

        // Add signals to state
        for signal in &signals {
            self.state.add_event(signal.clone().into());
        }

        // Return signals
        SignalSnapshot::new(timestamp.to_owned(), signals)
    }
}
