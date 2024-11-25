use std::sync::Arc;

use arkin_core::PubSub;

use crate::{Executor, ExecutorConfig, ExecutorTypeConfig};

use super::BacktestExecutorBuilder;

pub struct ExecutorFactory {}

impl ExecutorFactory {
    pub fn from_config(config: &ExecutorConfig, pubsub: Arc<PubSub>) -> Arc<dyn Executor> {
        let executor: Arc<dyn Executor> = match &config.executor {
            ExecutorTypeConfig::Simulation(_c) => Arc::new(
                BacktestExecutorBuilder::default()
                    .pubsub(pubsub)
                    .build()
                    .expect("Failed to build BacktestExecutor"),
            ),
            ExecutorTypeConfig::Binance(_c) => unimplemented!(),
        };

        executor
    }
}
