use arkin_common::prelude::*;

use crate::{config::AllocationManagerConfig, factory::AllocationFactory};

pub trait AllocationModule: Send + Sync {
    fn calculate(
        &self,
        market_snapshot: &MarketSnapshot,
        portfolio_snapshot: &PortfolioSnapshot,
        strategy_snapshot: &StrategySnapshot,
    ) -> Vec<ExecutionOrder>;
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
        market_snapshot: &MarketSnapshot,
        portfolio_snapshot: &PortfolioSnapshot,
        strategy_snapshot: &StrategySnapshot,
    ) -> AllocationSnapshot {
        // Calculate Allocations
        let allocations = self
            .allocation
            .calculate(market_snapshot, portfolio_snapshot, strategy_snapshot);

        AllocationSnapshot::new(market_snapshot.timestamp(), allocations)
    }
}
