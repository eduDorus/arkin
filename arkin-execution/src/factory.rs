use std::sync::Arc;

use crate::{ExecutionConfig, ExecutorFactory, OrderManager, OrderManagerType, SimpleOrderManagerBuilder};

pub struct ExecutionFactory {}

impl ExecutionFactory {
    pub fn from_config(config: &ExecutionConfig) -> Arc<dyn OrderManager> {
        let order_manager: Arc<dyn OrderManager> = match &config.order_manager {
            OrderManagerType::SimpleExecutor(c) => {
                let executor = ExecutorFactory::from_config(&c.executor);

                Arc::new(
                    SimpleOrderManagerBuilder::default()
                        .executor(executor)
                        .build()
                        .expect("Failed to build SimpleOrderManager"),
                )
            }
        };

        order_manager
    }
}
