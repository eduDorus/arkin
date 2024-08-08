use super::{factory::AllocationFactory, AllocationModule};
use crate::{
    config::AllocationManagerConfig,
    models::{Allocation, Signal},
};
use rayon::prelude::*;

pub struct AllocationManager {
    allocations: Vec<Box<dyn AllocationModule>>,
}

impl AllocationManager {
    pub fn from_config(config: &AllocationManagerConfig) -> Self {
        Self {
            allocations: AllocationFactory::from_config(&config.allocations),
        }
    }

    pub fn calculate(&self, signals: &[Signal]) -> Vec<Allocation> {
        self.allocations
            .par_iter()
            .map(|a| a.calculate(signals))
            .flat_map(|a| a)
            .collect::<Vec<_>>()
    }
}
