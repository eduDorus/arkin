use arkin_common::prelude::*;
use rayon::prelude::*;

use crate::{config::AllocationManagerConfig, factory::AllocationFactory};

pub trait AllocationModule: Send + Sync {
    fn calculate(
        &self,
        portfolio_snapshot: &PortfolioSnapshot,
        strategy_snapshot: &StrategySnapshot,
    ) -> Vec<Allocation>;
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

    pub fn calculate_allocations(
        &self,
        portfolio_snapshot: &PortfolioSnapshot,
        strategy_snapshot: &StrategySnapshot,
    ) -> Vec<Allocation> {
        self.allocations
            .par_iter()
            .map(|a| a.calculate(portfolio_snapshot, strategy_snapshot))
            .flat_map(|a| a)
            .collect::<Vec<_>>()
    }
}
