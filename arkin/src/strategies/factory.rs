use std::sync::Arc;

use crate::{config::StrategyConfig, state::StateManager};

use super::{crossover::CrossoverStrategy, spreader::Spreader, StrategyType};

pub struct StrategyFactory {}

impl StrategyFactory {
    pub fn from_config(state: Arc<StateManager>, configs: &Vec<StrategyConfig>) -> Vec<StrategyType> {
        let mut strategies = Vec::new();

        for config in configs {
            let strategy = match config {
                StrategyConfig::Crossover(c) => StrategyType::Crossover(CrossoverStrategy::new(state.to_owned(), c)),
                StrategyConfig::Spreader(c) => StrategyType::Spreader(Spreader::new(state.to_owned(), c)),
            };
            strategies.push(strategy);
        }

        strategies
    }
}
