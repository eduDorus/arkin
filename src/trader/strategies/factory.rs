use std::sync::Arc;

use crate::{config::StrategyConfig, state::StateManager};

use super::{crossover::CrossoverStrategy, spreader::Spreader, StrategyType};

pub struct StrategyFactory {}

impl StrategyFactory {
    pub fn from_config(state: Arc<StateManager>, config: &StrategyConfig) -> StrategyType {
        match config {
            StrategyConfig::Crossover(config) => StrategyType::WideQuoter(CrossoverStrategy::new(state, config)),
            StrategyConfig::Spreader(config) => StrategyType::Spreader(Spreader::new(state, config)),
        }
    }
}
