use arkin_common::prelude::*;

use crate::{config::AllocationManagerConfig, factory::AllocationFactory};

pub trait AllocationModule: Send + Sync {
    fn calculate(
        &self,
        portfolio_snapshot: &PortfolioSnapshot,
        strategy_snapshot: &StrategySnapshot,
    ) -> Vec<Allocation>;
}

pub struct AllocationManager {
    allocation: Box<dyn AllocationModule>,
}

impl AllocationManager {
    pub fn from_config(config: &AllocationManagerConfig) -> Self {
        Self {
            allocation: AllocationFactory::from_config(&config.module),
        }
    }

    pub fn process(
        &self,
        portfolio_snapshot: &PortfolioSnapshot,
        strategy_snapshot: &StrategySnapshot,
    ) -> AllocationSnapshot {
        let timestamp = strategy_snapshot.timestamp();

        // Calculate Allocations
        let allocations = self.allocation.calculate(portfolio_snapshot, strategy_snapshot);

        // Calculate Orders
        let orders = vec![];

        AllocationSnapshot::new(timestamp, allocations, orders)
    }
}
