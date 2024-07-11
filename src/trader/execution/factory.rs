use std::sync::Arc;

use crate::{config::ExecutionConfig, state::State};

use super::{binance::BinanceExecution, ExecutionType};

pub struct ExecutionFactory {}

impl ExecutionFactory {
    pub fn from_config(state: Arc<State>, config: &ExecutionConfig) -> ExecutionType {
        match config {
            ExecutionConfig::Forward(config) => ExecutionType::Binance(BinanceExecution::new(state, config)),
        }
    }
}
