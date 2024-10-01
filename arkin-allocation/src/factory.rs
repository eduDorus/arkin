use crate::{manager::AllocationModule, simple::SimpleAllocation, AllocationModuleConfig};

pub struct AllocationFactory {}

impl AllocationFactory {
    pub fn from_config(config: &AllocationModuleConfig) -> Box<dyn AllocationModule> {
        let allocation: Box<dyn AllocationModule> = match config {
            AllocationModuleConfig::Simple(c) => Box::new(SimpleAllocation::from_config(c)),
        };
        allocation
    }
}
