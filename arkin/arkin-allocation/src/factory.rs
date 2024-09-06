use crate::{equal::EqualAllocation, manager::AllocationModule, AllocationModuleConfig};

pub struct AllocationFactory {}

impl AllocationFactory {
    pub fn from_config(config: &AllocationModuleConfig) -> Box<dyn AllocationModule> {
        let allocation: Box<dyn AllocationModule> = match config {
            AllocationModuleConfig::Equal(c) => Box::new(EqualAllocation::from_config(c)),
        };
        allocation
    }
}
