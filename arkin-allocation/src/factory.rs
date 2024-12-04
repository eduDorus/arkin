use std::sync::Arc;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use arkin_portfolio::prelude::*;

use crate::{AllocationOptim, AllocationOptimConfig, AllocationTypeConfig, LimitedAllocationOptim};

pub struct AllocationFactory {}

impl AllocationFactory {
    pub fn from_config(
        config: &AllocationOptimConfig,
        pubsub: Arc<PubSub>,
        persistance: Arc<PersistenceService>,
        portfolio: Arc<dyn Accounting>,
    ) -> Arc<dyn AllocationOptim> {
        let allocation: Arc<dyn AllocationOptim> = match &config.allocation_optim {
            AllocationTypeConfig::Limited(c) => Arc::new(
                LimitedAllocationOptim::builder()
                    .pubsub(pubsub.clone())
                    .persistence(persistance)
                    .portfolio(portfolio)
                    .max_allocation(c.max_allocation)
                    .max_allocation_per_signal(c.max_allocation_per_signal)
                    .reference_currency(test_usdt_asset())
                    .build(),
            ),
        };
        allocation
    }
}
