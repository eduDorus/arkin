use super::{factory::AllocationFactory, Allocation};
use crate::{
    config::AllocationManagerConfig,
    models::{AllocationEvent, Signal},
};
use rayon::prelude::*;

pub struct AllocationManager {
    allocations: Vec<Box<dyn Allocation>>,
}

impl AllocationManager {
    pub fn from_config(config: &AllocationManagerConfig) -> Self {
        Self {
            allocations: AllocationFactory::from_config(&config.allocations),
        }
    }

    pub fn calculate(&self, signals: &[Signal]) -> Vec<AllocationEvent> {
        self.allocations
            .par_iter()
            .map(|a| a.calculate(signals))
            .flat_map(|a| a)
            .collect::<Vec<_>>()
    }
}
