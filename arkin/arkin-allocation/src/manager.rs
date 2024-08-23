use arkin_common::prelude::*;
use rayon::prelude::*;
use time::OffsetDateTime;

use crate::{config::AllocationManagerConfig, factory::AllocationFactory};

pub trait AllocationModule: Send + Sync {
    fn calculate(&self, signals: &SignalSnapshot, positions: &PositionSnapshot) -> Vec<Allocation>;
}

pub struct AllocationManager {
    allocations: Vec<Box<dyn AllocationModule>>,
}

impl AllocationManager {
    pub fn from_config(config: &AllocationManagerConfig) -> Self {
        Self {
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

        AllocationSnapshot::new(timestamp.to_owned(), allocations, Vec::new())
    }
}
