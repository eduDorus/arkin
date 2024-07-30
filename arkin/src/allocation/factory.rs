use std::sync::Arc;

use crate::{config::AllocationConfig, state::State};

use super::{limited::LimitedAllocation, AllocationType};

pub struct AllocationFactory {}

impl AllocationFactory {
    pub fn from_config(state: Arc<State>, config: &AllocationConfig) -> AllocationType {
        match config {
            AllocationConfig::Limited(c) => AllocationType::Limited(LimitedAllocation::new(state, c)),
        }
    }
}
