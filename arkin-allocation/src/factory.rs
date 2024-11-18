use std::sync::Arc;

use crate::{AllocationModuleConfig, AllocationOptim, LimitedAllocationOptimBuilder};

pub struct AllocationFactory {}

impl AllocationFactory {
    pub fn from_config(config: &AllocationModuleConfig) -> Arc<dyn AllocationOptim> {
        let allocation: Arc<dyn AllocationOptim> = match config {
            AllocationModuleConfig::Simple(c) => Arc::new(
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
