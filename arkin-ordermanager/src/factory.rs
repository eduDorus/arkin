use std::sync::Arc;

use arkin_core::prelude::*;

use crate::{DefaultOrderManager, OrderManagerService, OrderManagersConfig};

pub struct OrderManagerFactory {}

impl OrderManagerFactory {
    pub fn init(pubsub: Arc<PubSub>) -> Arc<dyn OrderManagerService> {
        let config = load::<OrderManagersConfig>();

        order_manager
    }
}
