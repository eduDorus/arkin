use std::sync::Arc;

use rust_decimal::Decimal;

use crate::{config::LimitedAllocationConfig, state::State};

#[derive(Clone)]
pub struct LimitedAllocation {
    state: Arc<State>,
    max_allocation: Decimal,
}

impl LimitedAllocation {
    pub fn new(state: Arc<State>, config: &LimitedAllocationConfig) -> Self {
        LimitedAllocation {
            state,
            max_allocation: config.max_allocation,
        }
    }
}
