use std::sync::Arc;

use super::{equal::EqualAllocation, AllocationModule};
use crate::{config::AllocationConfig, state::StateManager};

pub struct AllocationFactory {}

impl AllocationFactory {
    pub fn from_config(state: Arc<StateManager>, configs: &[AllocationConfig]) -> Vec<Box<dyn AllocationModule>> {
        let mut allocations = Vec::new();

        configs.iter().for_each(|c| {
            let allocation: Box<dyn AllocationModule> = match &c {
                AllocationConfig::Equal(c) => Box::new(EqualAllocation::from_config(state.clone(), c)),
            };
            allocations.push(allocation);
        });

        allocations
    }
}
