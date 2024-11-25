use std::sync::Arc;

use arkin_core::PubSub;
use arkin_portfolio::Portfolio;

use crate::{OrderManager, OrderManagerConfig, OrderManagerType, SimpleOrderManagerBuilder};

pub struct ExecutionFactory {}

impl ExecutionFactory {
    pub fn from_config(
        config: &OrderManagerConfig,
        pubsub: Arc<PubSub>,
        portfolio: Arc<dyn Portfolio>,
    ) -> Arc<dyn OrderManager> {
        let order_manager: Arc<dyn OrderManager> = match &config.order_manager {
            OrderManagerType::SimpleExecutor => Arc::new(
                SimpleOrderManagerBuilder::default()
                    .pubsub(pubsub)
                    .portfolio(portfolio)
                    .build()
                    .expect("Failed to build SimpleOrderManager"),
            ),
        };

        order_manager
    }
}
