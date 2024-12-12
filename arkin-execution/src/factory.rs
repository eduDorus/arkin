use std::sync::Arc;

use arkin_core::PubSub;

use crate::{OrderManager, OrderManagerConfig, OrderManagerType, SimpleOrderManager};

pub struct ExecutionFactory {}

impl ExecutionFactory {
    pub fn from_config(config: &OrderManagerConfig, pubsub: Arc<PubSub>) -> Arc<dyn OrderManager> {
        let order_manager: Arc<dyn OrderManager> = match &config.order_manager {
            OrderManagerType::SimpleExecutor => Arc::new(SimpleOrderManager::builder().pubsub(pubsub).build()),
        };

        order_manager
    }
}
