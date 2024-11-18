use std::sync::Arc;

use crate::{Executor, ExecutorTypeConfig};

pub struct ExecutorFactory {}

impl ExecutorFactory {
    pub fn from_config(config: &ExecutorTypeConfig) -> Arc<dyn Executor> {
        let executor: Arc<dyn Executor> = match config {
            ExecutorTypeConfig::Simulation(_c) => Arc::new(
                crate::SimulationExecutorBuilder::default()
                    .build()
                    .expect("Failed to build SimulationExecutor"),
            ),
            ExecutorTypeConfig::Binance(_c) => unimplemented!(),
        };

        executor
    }
}
