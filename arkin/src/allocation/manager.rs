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

    pub fn calculate(&self, signals: Vec<Signal>) -> Vec<AllocationEvent> {
        self.allocations
            .par_iter()
            .map(|a| {
                // Filter signals
                let signals = signals
                    .iter()
                    .filter(|s| a.strategies().contains(&s.strategy_id))
                    .cloned()
                    .collect::<Vec<_>>();

                a.calculate(signals)
            })
            .flat_map(|a| a)
            .collect::<Vec<_>>()
    }
}
