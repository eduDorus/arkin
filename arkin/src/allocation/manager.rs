use std::sync::Arc;

use super::{factory::AllocationFactory, AllocationModule};
use crate::{
    config::AllocationManagerConfig,
    models::{ExecutionOrder, Signal},
    state::StateManager,
};
use rayon::prelude::*;

pub struct AllocationManager {
    allocations: Vec<Box<dyn AllocationModule>>,
}

impl AllocationManager {
    pub fn from_config(state: Arc<StateManager>, config: &AllocationManagerConfig) -> Self {
        Self {
            allocations: AllocationFactory::from_config(state, &config.allocations),
        }
    }

    pub fn calculate(&self, signals: &[Signal]) -> Vec<ExecutionOrder> {
        self.allocations
            .par_iter()
            .map(|a| a.calculate(signals))
            .flat_map(|a| a)
            .collect::<Vec<_>>()
    }
}
