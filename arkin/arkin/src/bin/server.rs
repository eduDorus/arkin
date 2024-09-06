use std::time::Duration;

use mimalloc::MiMalloc;
use time::macros::datetime;
use tokio_rustls::rustls::crypto::aws_lc_rs;
use tokio_rustls::rustls::crypto::CryptoProvider;
use tracing::info;

use arkin_allocation::prelude::*;
use arkin_common::prelude::*;
use arkin_engine::prelude::*;
use arkin_execution::prelude::*;
use arkin_insights::prelude::*;
use arkin_market::prelude::*;
use arkin_persistance::prelude::*;
use arkin_portfolio::prelude::*;
use arkin_strategies::prelude::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();
    info!("Starting Arkin 🚀");

    // Install the default CryptoProvider
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");

    let start = datetime!(2024-07-01 00:00).assume_utc();
    let end = datetime!(2024-07-01 01:00).assume_utc();
    let frequency_secs = Duration::from_secs(5);

    // Market Manager
    let market_manager = MarketManager::from_config(&load::<MarketConfig>().market_manager);

    // Load data
    info!("Loading trades and ticks");
    let database = DBManager::from_config(&load::<PersistanceConfig>().database);
    let ticks = database.read_ticks(&start, &end).await;
    let trades = database.read_trades(&start, &end).await;
    info!("Loaded {} trades and {} ticks", trades.len(), ticks.len());
    market_manager.insert_batch(ticks.into_iter().map(|v| v.into()).collect());
    market_manager.insert_batch(trades.into_iter().map(|v| v.into()).collect());

    // Create the engine
    let engine = EngineBuilder::default()
        .portfolio_manager(PortfolioManager::from_config(&load::<PortfolioConfig>().portfolio_manager))
        .market_manager(market_manager)
        .insights_manager(InsightsManager::from_config(&load::<InsightsConfig>().insights_manager))
        .strategy_manager(StrategyManager::from_config(&load::<StrategyConfig>().strategy_manager))
        .allocation_manager(AllocationManager::from_config(&load::<AllocationConfig>().allocation_manager))
        .execution_manager(ExecutionManager::from_config(&load::<ExecutionConfig>().execution_manager))
        .build()
        .expect("Failed to build engine");

    // Run the engine
    engine.backtest(start, end, frequency_secs);
}
