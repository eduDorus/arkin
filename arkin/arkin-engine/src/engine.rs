use std::time::Duration;

use arkin_allocation::AllocationManager;
use arkin_common::prelude::Clock;
use arkin_execution::ExecutionManager;
use arkin_insights::InsightsManager;
use arkin_market::MarketManager;
use arkin_portfolio::PortfolioManager;
use arkin_strategies::StrategyManager;
use time::OffsetDateTime;
use tracing::info;

pub struct Engine {
    market_manager: MarketManager,
    portfolio_manager: PortfolioManager,
    insights_manager: InsightsManager,
    strategy_manager: StrategyManager,
    allocation_manager: AllocationManager,
    execution_manager: ExecutionManager,
}

impl Engine {
    pub fn new(
        market_manager: MarketManager,
        portfolio_manager: PortfolioManager,
        insights_manager: InsightsManager,
        strategy_manager: StrategyManager,
        allocation_manager: AllocationManager,
        execution_manager: ExecutionManager,
    ) -> Self {
        Self {
            market_manager,
            portfolio_manager,
            insights_manager,
            strategy_manager,
            allocation_manager,
            execution_manager,
        }
    }

    pub fn backtest(&self, start: OffsetDateTime, end: OffsetDateTime, frequency_secs: Duration) {
        let mut clock = Clock::new(start, end, frequency_secs);

        while let Some(timestamp) = clock.next() {
            info!("----------------- {:?} -----------------", timestamp);
            let market_snapshot = self.market_manager.snapshot(&timestamp, frequency_secs);

            // insights_manager.insert_batch(trade_features.to_owned());
            // insights_manager.insert_batch(tick_features.to_owned());

            // snapshot.add_insights(insights_manager.calculate(&snapshot));
        }
    }
}
