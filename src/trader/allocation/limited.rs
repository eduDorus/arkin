use std::sync::Arc;

use rust_decimal::Decimal;

use crate::{config::LimitedAllocationConfig, state::StateManager};

#[derive(Clone)]
pub struct LimitedAllocation {
    state: Arc<StateManager>,
    max_allocation: Decimal,
}

impl LimitedAllocation {
    pub fn new(state: Arc<StateManager>, config: &LimitedAllocationConfig) -> Self {
        LimitedAllocation {
            state,
            max_allocation: config.max_allocation,
        }
    }
}
