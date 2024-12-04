use std::sync::Arc;

use arkin_core::PubSub;

use crate::{Executor, ExecutorConfig, ExecutorTypeConfig};

use super::SimulationExecutor;

pub struct ExecutorFactory {}

impl ExecutorFactory {
    pub fn from_config(config: &ExecutorConfig, pubsub: Arc<PubSub>) -> Arc<dyn Executor> {
        let executor: Arc<dyn Executor> = match &config.executor {
            ExecutorTypeConfig::Simulation(_c) => Arc::new(SimulationExecutor::builder().pubsub(pubsub).build()),
            ExecutorTypeConfig::Binance(_c) => unimplemented!(),
        };

        executor
    }
}
