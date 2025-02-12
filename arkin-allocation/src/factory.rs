use std::sync::Arc;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use arkin_portfolio::prelude::*;

use crate::{AllocationOptimConfig, AllocationService, AllocationTypeConfig, LimitedAllocationOptim};

pub struct AllocationFactory {}

impl AllocationFactory {
    pub fn init(
        pubsub: Arc<PubSub>,
        persistance: Arc<PersistenceService>,
        portfolio: Arc<dyn Accounting>,
    ) -> Arc<dyn AllocationService> {
        let config = load::<AllocationOptimConfig>();
        let allocation: Arc<dyn AllocationService> = match &config.allocation_optim {
            AllocationTypeConfig::Limited(c) => Arc::new(
                LimitedAllocationOptim::builder()
                    .pubsub(pubsub.clone())
                    .persistence(persistance)
                    .portfolio(portfolio)
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
