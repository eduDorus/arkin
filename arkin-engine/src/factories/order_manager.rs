use std::sync::Arc;

use arkin_core::prelude::*;
use arkin_ordermanager::prelude::*;

pub struct OrderManagerFactory {}

impl OrderManagerFactory {
    pub fn init(pubsub: Arc<PubSub>) -> Arc<dyn OrderManagerService> {
        let order_manager: Arc<dyn OrderManagerService> =
            Arc::new(DefaultOrderManager::builder().pubsub(pubsub.clone()).build());

        order_manager
    }
}
