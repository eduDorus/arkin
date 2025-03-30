use std::sync::Arc;

use arkin_accounting::prelude::*;
use arkin_allocation::prelude::*;
use arkin_core::prelude::*;
use arkin_persistence::prelude::*;

use crate::{AllocationOptimConfig, AllocationTypeConfig};

pub struct AllocationFactory {}

impl AllocationFactory {
    pub async fn init(
        pubsub: Arc<PubSub>,
        persistance: Arc<PersistenceService>,
        accounting: Arc<dyn Accounting>,
    ) -> Arc<dyn AllocationService> {
        let config = load::<AllocationOptimConfig>();
        let allocation: Arc<dyn AllocationService> = match &config.allocation_optim {
            AllocationTypeConfig::Limited(c) => Arc::new(
                SignalAllocationOptim::builder()
                    .pubsub(pubsub.handle().await)
                    .persistence(persistance)
                    .accounting(accounting)
                    .leverage(c.leverage)
                    .min_trade_value(c.min_trade_value)
                    .allocation_feature_id(c.allocation_feature_id.clone())
                    .reference_currency(test_usdt_asset())
                    .build(),
            ),
        };
        allocation
    }
}
