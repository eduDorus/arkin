use super::{equal::EqualAllocation, Allocation};
use crate::config::AllocationConfig;

pub struct AllocationFactory {}

impl AllocationFactory {
    pub fn from_config(configs: &[AllocationConfig]) -> Vec<Box<dyn Allocation>> {
        let mut allocations = Vec::new();

        configs.iter().for_each(|c| {
            let allocation: Box<dyn Allocation> = match &c {
                AllocationConfig::Equal(c) => Box::new(EqualAllocation::from_config(c)),
            };
            allocations.push(allocation);
        });

        allocations
    }
}
