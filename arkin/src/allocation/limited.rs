use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;
use tracing::info;

use crate::{config::LimitedAllocationConfig, state::State};

use super::Allocation;

#[derive(Clone)]
#[allow(unused)]
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

#[async_trait]
impl Allocation for LimitedAllocation {
    async fn start(&self) {
        info!("Starting limited allocation...");
    }
}
