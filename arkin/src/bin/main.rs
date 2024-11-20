use std::sync::Arc;

use arkin_portfolio::{PortfolioConfig, PortfolioFactory};
use mimalloc::MiMalloc;
use tokio_rustls::rustls::crypto::{aws_lc_rs, CryptoProvider};
use tracing::{error, info};

use arkin_allocation::prelude::*;
use arkin_core::prelude::*;
use arkin_engine::prelude::*;
use arkin_execution::prelude::*;
use arkin_ingestors::prelude::*;
use arkin_insights::prelude::*;
use arkin_persistence::prelude::*;
use arkin_strategies::prelude::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");
    info!("Starting Arkin Order Manager 🚀");

    let config = load::<PersistenceConfig>();
    let persistence_service = Arc::new(PersistenceService::from_config(&config));

    let config = load::<PortfolioConfig>();
    let portfolio = PortfolioFactory::from_config(&config);

    let config = load::<ExecutionConfig>();
    let order_manager = ExecutionFactory::from_config(&config, portfolio.clone());

    let config = load::<AllocationOptimConfig>();
    let allocation = AllocationFactory::from_config(&config, portfolio.clone());

    let config = load::<InsightsConfig>();
    let insights = Arc::new(InsightsService::from_config(
        &config.insights_service,
        persistence_service.clone(),
    ));

    let config = load::<StrategyConfig>();
    let strategy = StrategyFactory::from_config(&config).pop().expect("No strategy found");

    let config = load::<IngestorsConfig>();
    let ingestors = IngestorFactory::from_config(&config, persistence_service.clone());

    // Work around for fetching instruments
    let venue_symbols = vec!["BTCUSDT", "ETHUSDT", "SOLUSDT"];
    let mut instruments = vec![];
    for symbol in venue_symbols {
        match persistence_service.read_instrument_by_venue_symbol(symbol.to_string()).await {
            Ok(instr) => instruments.push(instr),
            Err(e) => error!("Failed to read instrument {}: {}", symbol, e),
        }
    }

    let engine = SingleStrategyEngineBuilder::default()
        .instruments(instruments)
        .persistor(persistence_service)
        .portfolio(portfolio)
        .ingestors(ingestors)
        .insights(insights)
        .strategy(strategy)
        .allocation_optim(allocation)
        .order_manager(order_manager)
        .build()
        .expect("Failed to build DefaultEngine");

    engine.start().await.expect("Failed to start engine");

    info!("Waiting for shutdown to complete...");
    engine.stop().await.expect("Failed to stop engine");
    info!("Shutdown complete");
}
