use std::time::Duration;

use arkin_allocation::AllocationManager;
use arkin_core::prelude::Clock;
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
        self.portfolio_manager.print_stats();
        // self.portfolio_manager.print_trades()
    }

    fn run_cycle(&self, timestamp: OffsetDateTime, frequency_secs: Duration) {
        // Snapshot the market and portfolio
        let market_snapshot = self.market_manager.snapshot(&timestamp, frequency_secs);
        // for data in market_snapshot.insights() {
        //     info!("Market data: {}", data);
        // }
        let portfolio_snapshot = self.portfolio_manager.snapshot(&timestamp);

        // Process the insights
        let insights_snapshot = self.insights_manager.process(&market_snapshot);
        // for data in insights_snapshot.insights() {
        //     info!("Insights data: {}", data);
        // }
        let strategy_snapshot = self.strategy_manager.process(&insights_snapshot);
        for signal in &strategy_snapshot.signals {
            info!("Signal: {}", signal);
        }

        let allocation_snapshot =
            self.allocation_manager
                .process(&market_snapshot, &portfolio_snapshot, &strategy_snapshot);
        for order in &allocation_snapshot.orders {
            info!("Order: {}", order);
        }

        let fills = self.execution_manager.process_backtest(&allocation_snapshot, &market_snapshot);
        for fill in fills {
            info!("Fill: {}", fill);
            self.portfolio_manager.update_position_from_fill(fill.clone());
        }
    }
}
