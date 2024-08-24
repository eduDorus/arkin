use crate::{equal::EqualAllocation, manager::AllocationModule, AllocationModuleConfig};

pub struct AllocationFactory {}

impl AllocationFactory {
    pub fn from_config(configs: &[AllocationModuleConfig]) -> Vec<Box<dyn AllocationModule>> {
        let mut allocations = Vec::new();

        configs.iter().for_each(|c| {
            let allocation: Box<dyn AllocationModule> = match &c {
                AllocationModuleConfig::Equal(c) => Box::new(EqualAllocation::from_config(c)),
            };
            allocations.push(allocation);
        });

        allocations
    }
}
