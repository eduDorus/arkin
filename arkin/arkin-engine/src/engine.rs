use std::time::Duration;

use arkin_allocation::AllocationManager;
use arkin_common::prelude::Clock;
use arkin_execution::ExecutionManager;
use arkin_insights::InsightsManager;
use arkin_market::MarketManager;
use arkin_portfolio::PortfolioManager;
use arkin_strategies::StrategyManager;
use derive_builder::Builder;
use time::OffsetDateTime;
use tracing::info;

#[derive(Builder)]
#[builder(pattern = "owned")]
pub struct Engine {
    market_manager: MarketManager,
    portfolio_manager: PortfolioManager,
    insights_manager: InsightsManager,
    strategy_manager: StrategyManager,
    allocation_manager: AllocationManager,
    execution_manager: ExecutionManager,
}

impl Engine {
    pub fn backtest(&self, start: OffsetDateTime, end: OffsetDateTime, frequency_secs: Duration) {
        let mut clock = Clock::new(start, end, frequency_secs);

        while let Some(timestamp) = clock.next() {
            info!("----------------- {:?} -----------------", timestamp);
            self.run_cycle(timestamp, frequency_secs);
        }
    }

    fn run_cycle(&self, timestamp: OffsetDateTime, frequency_secs: Duration) {
        // Snapshot the market and portfolio
        let market_snapshot = self.market_manager.snapshot(&timestamp, frequency_secs);
        let portfolio_snapshot = self.portfolio_manager.snapshot(&timestamp);

        // Process the insights
        let insights_snapshot = self.insights_manager.process(&market_snapshot);
        let strategy_snapshot = self.strategy_manager.process(&insights_snapshot);
        let allocation_snapshot = self.allocation_manager.process(&portfolio_snapshot, &strategy_snapshot);
        self.execution_manager.process(&allocation_snapshot);
    }
}
