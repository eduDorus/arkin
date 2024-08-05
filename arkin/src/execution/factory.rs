use super::{binance::BinanceExecution, ExecutionType};
use crate::{config::ExecutionConfig, state::State};
use std::sync::Arc;

pub struct ExecutionFactory {}

impl ExecutionFactory {
    pub fn from_config(state: Arc<State>, configs: &[ExecutionConfig]) -> Vec<ExecutionType> {
        let mut executors = Vec::new();

        for config in configs {
            match config {
                ExecutionConfig::Binance(config) => {
                    executors.push(ExecutionType::Binance(BinanceExecution::new(state.to_owned(), config)))
                }
            }
        }

        executors
    }
}
