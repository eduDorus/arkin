use std::sync::Arc;

use rust_decimal::Decimal;
use tracing::info;

use crate::{config::LimitedAllocationConfig, state::StateManager};

use super::Allocation;

#[derive(Clone)]
#[allow(unused)]
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

impl Allocation for LimitedAllocation {
    async fn start(&self) {
        info!("Starting limited allocation...");
    }
}
