use mimalloc::MiMalloc;
use tokio_rustls::rustls::crypto::aws_lc_rs;
use tokio_rustls::rustls::crypto::CryptoProvider;
use tracing::info;

use arkin_allocation::prelude::*;
use arkin_common::prelude::*;
use arkin_engine::prelude::*;
use arkin_execution::prelude::*;
use arkin_insights::prelude::*;
use arkin_market::prelude::*;
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

    // Create the engine
    let _engine = EngineBuilder::default()
        .portfolio_manager(PortfolioManager::from_config(&load::<PortfolioConfig>().portfolio_manager))
        .market_manager(MarketManager::from_config(&load::<MarketConfig>().market_manager))
        .insights_manager(InsightsManager::from_config(&load::<InsightsConfig>().insights_manager))
        .strategy_manager(StrategyManager::from_config(&load::<StrategyConfig>().strategy_manager))
        .allocation_manager(AllocationManager::from_config(&load::<AllocationConfig>().allocation_manager))
        .execution_manager(ExecutionManager::from_config(&load::<ExecutionConfig>().execution_manager))
        .build()
        .expect("Failed to build engine");
}
