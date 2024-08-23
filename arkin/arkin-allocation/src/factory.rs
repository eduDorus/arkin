use crate::{config::AllocationConfig, equal::EqualAllocation, manager::AllocationModule};

pub struct AllocationFactory {}

impl AllocationFactory {
    pub fn from_config(configs: &[AllocationConfig]) -> Vec<Box<dyn AllocationModule>> {
        let mut allocations = Vec::new();

        configs.iter().for_each(|c| {
            let allocation: Box<dyn AllocationModule> = match &c {
                AllocationConfig::Equal(c) => Box::new(EqualAllocation::from_config(c)),
            };
            allocations.push(allocation);
        });

        allocations
    }
}
