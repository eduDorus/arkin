use std::time::Duration;

use rstest::*;
use time::macros::datetime;
use tracing::info;

use arkin_allocation::prelude::*;
use arkin_common::prelude::*;
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
    portfolio_manager: PortfolioManager,
    insights_manager: InsightsManager,
    strategy_manager: StrategyManager,
    allocation_manager: AllocationManager,
) {
    info!("Starting end-to-end test");

    let start = datetime!(2024-07-01 00:00).assume_utc();
    let end = datetime!(2024-07-01 01:00).assume_utc();
    let frequency_secs = Duration::from_secs(5);
    info!("Start: {}, End: {}", start, end);

    // Load trades
    info!("Loading trades and ticks");
    let ticks = database.read_ticks(&start, &end).await;
    let trades = database.read_trades(&start, &end).await;
    info!("Loaded {} trades and {} ticks", trades.len(), ticks.len());

    // insights
    // let trade_features = trades.into_iter().flat_map::<Vec<Insight>, _>(|t| t.into()).collect();
    // let tick_features = ticks.into_iter().flat_map::<Vec<Insight>, _>(|t| t.into()).collect();
    market_manager.insert_batch(ticks.into_iter().map(|v| v.into()).collect());
    market_manager.insert_batch(trades.into_iter().map(|v| v.into()).collect());

    let (mut timestamp, intervals) = calculate_intervals(&start, &end, frequency_secs);
    for _ in 0..intervals {
        info!("----------------- {:?} -----------------", timestamp);
        // Take a snapshot of the market and positions
        // let mut snapshot = SnapshotBuilder::default()
        //     .event_time(timestamp)
        //     .market(market_manager.snapshot(&timestamp, frequency_secs))
        //     .portfolio(portfolio_manager.snapshot(&timestamp))
        //     .build()
        //     .unwrap();

        // // Calculate insights
        // let trade_features = &snapshot
        //     .market
        //     .trades
        //     .iter()
        //     .flat_map(|(_, v)| v)
        //     .map::<Vec<Insight>, _>(|t| t.clone().into())
        //     .flatten()
        //     .collect::<Vec<_>>();

        // let tick_features = &snapshot
        //     .market
        //     .ticks
        //     .iter()
        //     .flat_map(|(_, v)| v)
        //     .map::<Vec<Insight>, _>(|t| t.clone().into())
        //     .flatten()
        //     .collect::<Vec<_>>();

        // insights_manager.insert_batch(trade_features.to_owned());
        // insights_manager.insert_batch(tick_features.to_owned());

        // snapshot.add_insights(insights_manager.calculate(&snapshot));
        // for metric in &snapshot.insights {
        //     info!("Insight: {}", metric);
        // }

        // // Calculate signals
        // snapshot.add_signals(strategy_manager.calculate(&snapshot));
        // for signal in &snapshot.signals {
        //     info!("Signal: {}", signal);
        // }

        // // Calculate allocations
        // snapshot.add_allocations(allocation_manager.calculate_allocations(&snapshot));
        // for allocation in &snapshot.allocations {
        //     info!("Allocation: {}", allocation);
        // }

        // Increase timestamp
        timestamp += frequency_secs;
    }
}
