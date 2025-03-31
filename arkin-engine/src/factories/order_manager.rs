use std::sync::Arc;

use arkin_core::prelude::*;
use arkin_ordermanager::prelude::*;

pub struct OrderManagerFactory {}

impl OrderManagerFactory {
    pub async fn init(pubsub: Arc<PubSub>) -> Arc<dyn OrderManagerService> {
        let order_manager: Arc<dyn OrderManagerService> = Arc::new(
            DefaultOrderManager::builder()
                .pubsub(pubsub.handle("OrderManager").await)
                .build(),
        );

        order_manager
    }
}
