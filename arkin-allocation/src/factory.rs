use std::sync::Arc;

use arkin_portfolio::prelude::*;

use crate::{AllocationOptim, AllocationOptimConfig, AllocationTypeConfig, LimitedAllocationOptimBuilder};

pub struct AllocationFactory {}

impl AllocationFactory {
    pub fn from_config(config: &AllocationOptimConfig, portfolio: Arc<dyn Portfolio>) -> Arc<dyn AllocationOptim> {
        let allocation: Arc<dyn AllocationOptim> = match &config.allocation_optim {
            AllocationTypeConfig::Limited(c) => Arc::new(
                LimitedAllocationOptimBuilder::default()
                    .portfolio(portfolio)
                    .max_allocation(c.max_allocation)
                    .max_allocation_per_signal(c.max_allocation_per_signal)
                    .build()
                    .expect("Failed to build LimitedAllocationOptim"),
            ),
        };
        allocation
    }
}
