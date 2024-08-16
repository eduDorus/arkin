use std::sync::Arc;

use super::{factory::AllocationFactory, AllocationModule};
use crate::{
    config::AllocationManagerConfig,
    models::{AllocationSnapshot, MarketSnapshot, PositionSnapshot, SignalSnapshot},
    state::StateManager,
};
use rayon::prelude::*;
use time::OffsetDateTime;

pub struct AllocationManager {
    state: Arc<StateManager>,
    allocations: Vec<Box<dyn AllocationModule>>,
}

impl AllocationManager {
    pub fn from_config(state: Arc<StateManager>, config: &AllocationManagerConfig) -> Self {
        Self {
            state,
            allocations: AllocationFactory::from_config(&config.allocations),
        }
    }

    pub fn calculate(
        &self,
        timestamp: &OffsetDateTime,
        signals: &SignalSnapshot,
        _market: &MarketSnapshot,
        positions: &PositionSnapshot,
    ) -> AllocationSnapshot {
        // Calculate allocations for each module
        let allocations = self
            .allocations
            .par_iter()
            .map(|a| a.calculate(signals, positions))
            .flat_map(|a| a)
            .collect::<Vec<_>>();

        // Generate execution orders
        // TODO: Implement execution order generation

        // Add allocations to state
        for allocation in &allocations {
            self.state.add_event(allocation.clone().into());
        }

        AllocationSnapshot::new(timestamp.to_owned(), allocations, Vec::new())
    }
}
