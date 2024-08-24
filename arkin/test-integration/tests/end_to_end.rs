use std::time::Duration;

use rstest::*;
use time::macros::datetime;
use tracing::info;

use arkin_common::prelude::*;
use arkin_insights::prelude::*;
use arkin_persistance::prelude::*;
use arkin_strategies::prelude::*;
use test_utils::prelude::*;

#[rstest]
#[tokio::test]
async fn test_end_to_end(database: DBManager, insights_manager: InsightsManager, strategy_manager: StrategyManager) {
    info!("Starting end-to-end test");

    let start = datetime!(2024-07-01 00:00).assume_utc();
    let end = datetime!(2024-07-01 01:00).assume_utc();
    let frequency_secs = 1;
    info!("Start: {}, End: {}", start, end);

    // Load trades
    info!("Loading trades and ticks");
    let trades = database.read_ticks(&start, &end).await;
    let ticks = database.read_trades(&start, &end).await;
    // assert_eq!(trades.len(), 109151);
    // assert_eq!(ticks.len(), 8706);
    info!("Loaded {} trades and {} ticks", trades.len(), ticks.len());

    // insights
    let trade_features = trades.into_iter().flat_map::<Vec<Feature>, _>(|t| t.into()).collect();
    let tick_features = ticks.into_iter().flat_map::<Vec<Feature>, _>(|t| t.into()).collect();
    insights_manager.insert_batch(trade_features);
    insights_manager.insert_batch(tick_features);

    let (mut timestamp, intervals) = calculate_intervals(&start, &end, frequency_secs);
    for _ in 0..intervals {
        info!("----------------- {:?} -----------------", timestamp);
        let insights = insights_manager.calculate(&timestamp);
        for metric in &insights.metrics {
            info!("Insight: {}", metric);
        }

        let signals = strategy_manager.calculate(&timestamp, &insights);
        for signal in &signals.signals {
            info!("Signal: {}", signal);
        }

        timestamp += Duration::from_secs(frequency_secs);
    }
}
