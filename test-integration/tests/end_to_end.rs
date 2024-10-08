use std::time::Duration;

use rstest::*;
use time::macros::datetime;
use tracing::info;

use arkin_allocation::prelude::*;
use arkin_execution::prelude::*;
use arkin_insights::prelude::*;
use arkin_market::prelude::*;
use arkin_persistance::prelude::*;
use arkin_portfolio::prelude::*;
use arkin_strategies::prelude::*;
use test_utils::prelude::*;

#[rstest]
#[tokio::test]
async fn test_end_to_end(
    database: DBManager,
    market_manager: MarketManager,
    _portfolio_manager: PortfolioManager,
    _insights_manager: InsightsManager,
    _strategy_manager: StrategyManager,
    _allocation_manager: AllocationManager,
    _execution_manager: ExecutionManager,
) {
    info!("Starting end-to-end test");

    let start = datetime!(2024-07-01 00:00).assume_utc();
    let end = datetime!(2024-07-01 01:00).assume_utc();
    let _frequency_secs = Duration::from_secs(5);

    // Load data
    info!("Loading trades and ticks");
    let ticks = database.read_ticks(&start, &end).await;
    let trades = database.read_trades(&start, &end).await;
    info!("Loaded {} trades and {} ticks", trades.len(), ticks.len());
    market_manager.insert_batch(ticks.into_iter().map(|v| v.into()).collect());
    market_manager.insert_batch(trades.into_iter().map(|v| v.into()).collect());
}
