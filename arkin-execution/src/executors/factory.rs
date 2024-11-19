use std::sync::Arc;

use crate::{Executor, ExecutorTypeConfig};

use super::BacktestExecutorBuilder;

pub struct ExecutorFactory {}

impl ExecutorFactory {
    pub fn from_config(config: &ExecutorTypeConfig) -> Arc<dyn Executor> {
        let executor: Arc<dyn Executor> = match config {
            ExecutorTypeConfig::Simulation(_c) => Arc::new(
                BacktestExecutorBuilder::default()
                    .build()
                    .expect("Failed to build BacktestExecutor"),
            ),
            ExecutorTypeConfig::Binance(_c) => unimplemented!(),
        };

        executor
    }
}
