use std::sync::Arc;

use crate::{AllocationOptim, AllocationOptimConfig, AllocationTypeConfig, LimitedAllocationOptimBuilder};

pub struct AllocationFactory {}

impl AllocationFactory {
    pub fn from_config(config: &AllocationOptimConfig) -> Arc<dyn AllocationOptim> {
        let allocation: Arc<dyn AllocationOptim> = match &config.allocation_optim {
            AllocationTypeConfig::Limited(c) => Arc::new(
                LimitedAllocationOptimBuilder::default()
                    .max_allocation(c.max_allocation)
                    .max_allocation_per_signal(c.max_allocation_per_signal)
                    .build()
                    .expect("Failed to build LimitedAllocationOptim"),
            ),
        };
        allocation
    }
}
