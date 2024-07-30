use std::sync::Arc;

use crate::{config::ExecutionConfig, state::State};

use super::{backtest::BacktestExecution, binance::BinanceExecution, ExecutionType};

pub struct ExecutionFactory {}

impl ExecutionFactory {
    pub fn from_config(state: Arc<State>, configs: &[ExecutionConfig]) -> Vec<ExecutionType> {
        let mut executors = Vec::new();

        for config in configs {
            match config {
                ExecutionConfig::Backtest(config) => {
                    executors.push(ExecutionType::Backtest(BacktestExecution::new(state.to_owned(), config)))
                }
                ExecutionConfig::Binance(config) => {
                    executors.push(ExecutionType::Binance(BinanceExecution::new(state.to_owned(), config)))
                }
            }
        }

        executors
    }
}
