use std::sync::Arc;

use crate::{config::StrategyConfig, state::StateManager};

use super::{spreader::Spreader, wide_quoter::WideQuoter, StrategyType};

pub struct StrategyFactory {}

impl StrategyFactory {
    pub fn from_config(state: Arc<StateManager>, config: &StrategyConfig) -> StrategyType {
        match config {
            StrategyConfig::WideQuoter(config) => StrategyType::WideQuoter(WideQuoter::new(state, config)),
            StrategyConfig::Spreader(config) => StrategyType::Spreader(Spreader::new(state, config)),
        }
    }
}
