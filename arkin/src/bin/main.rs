use std::sync::Arc;

use arkin_portfolio::DefaultPortfolioBuilder;
use mimalloc::MiMalloc;
use rust_decimal::prelude::*;
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

    let portfolio = Arc::new(
        DefaultPortfolioBuilder::default()
            .capital(Decimal::from_f64(10000.0).expect("Failed to build Decimal"))
            .leverage(Decimal::from_f64(1.0).expect("Failed to build Decimal"))
            .build()
            .expect("Failed to build DefaultPortfolio"),
    );

    let executor = Arc::new(
        SimulationExecutorBuilder::default()
            .build()
            .expect("Failed to build SimulationExecutor"),
    );

    let order_manager = Arc::new(
        DefaultOrderManagerBuilder::default()
            .executor(executor)
            .build()
            .expect("Failed to build OrderManager"),
    );

    let allocation = Arc::new(
        LimitedAllocationOptimBuilder::default()
            .max_allocation(Decimal::from_f64(0.8).expect("Failed to build Decimal"))
            .max_allocation_per_signal(Decimal::from_f64(0.1).expect("Failed to build Decimal"))
            .build()
            .expect("Failed to build LimitedAllocationOptim"),
    );

    let config = load::<InsightsConfig>();
    let insights = Arc::new(InsightsService::from_config(
        &config.insights_service,
        persistence_service.clone(),
    ));

    let crossover_strategy = Arc::new(
        CrossoverStrategyBuilder::default()
            .id(Arc::new("crossover".to_string()))
            .price_source(Arc::new("close".to_string()))
            .volume_source(Arc::new("volume".to_string()))
            .build()
            .expect("Failed to build CrossoverStrategy"),
    );

    let binance_ingestor = Arc::new(
        BinanceIngestorBuilder::default()
            .persistence_service(persistence_service.clone())
            .url("wss://fstream.binance.com/ws".parse().unwrap())
            .channels(vec!["btcusdt@aggTrade".to_string()])
            .connections_per_manager(2)
            .duplicate_lookback(10000)
            .build()
            .expect("Failed to build BinanceIngestor"),
    );

    let engine = DefaultEngineBuilder::default()
        .persistor(persistence_service)
        .portfolio(portfolio)
        .ingestors(vec![binance_ingestor])
        .insights(insights)
        .strategies(vec![crossover_strategy])
        .allocation_optim(allocation)
        .order_manager(order_manager)
        .build()
        .expect("Failed to build DefaultEngine");

    engine.start().await.expect("Failed to start engine");

    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            info!("Shutdown signal received...");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }
    info!("Waiting for shutdown to complete...");
    engine.stop().await.expect("Failed to stop engine");
    info!("Shutdown complete");
}
