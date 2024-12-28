use std::{sync::Arc, time::Duration};

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use time::{macros::format_description, OffsetDateTime, PrimitiveDateTime};
use tokio_rustls::rustls::crypto::{aws_lc_rs, CryptoProvider};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

use arkin_allocation::prelude::*;
use arkin_core::prelude::*;
use arkin_engine::prelude::*;
use arkin_execution::prelude::*;
use arkin_ingestors::prelude::*;
use arkin_insights::prelude::*;
use arkin_persistence::prelude::*;
use arkin_portfolio::prelude::*;

/// CLI application for X
#[derive(Parser)]
#[clap(
    name = "Arkin",
    version = "0.1.0",
    author = "Dorus Janssens",
    about = "Welcome to the world of Arkin!"
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Perform insights related operations
    Insights(InsightsArgs),

    /// Perform ingestors related operations
    #[clap(subcommand)]
    Ingestors(IngestorsCommands),

    /// Perform engine related operations
    Engine(EngineArgs),
}

#[derive(Args, Debug)]
struct InsightsArgs {
    /// Source of data (e.g., db)
    #[arg(long, short)]
    source: String,

    /// Destination format (e.g., parquet)
    #[arg(long, short)]
    dest: String,

    /// Start date in "YYYY-MM-DD HH:MM" format
    #[arg(long, short, value_parser = parse_datetime)]
    from: OffsetDateTime,

    /// End date in "YYYY-MM-DD HH:MM" format
    #[arg(long, short, value_parser = parse_datetime)]
    till: OffsetDateTime,

    /// Pipeline name (e.g., hft)
    #[arg(long, short, value_delimiter = ',')]
    instruments: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum IngestorsCommands {
    /// Configure and start Binance ingestor
    Binance(BinanceIngestorArgs),

    /// Configure and start Tardis ingestor
    Tardis(TardisIngestorArgs),
}

#[derive(Args, Debug)]
struct BinanceIngestorArgs {
    /// Configure the channels to subscribe to
    #[arg(long, short, value_delimiter = ',')]
    channels: Vec<String>,

    /// Configure the instruments to subscribe to
    #[arg(long, short, value_delimiter = ',')]
    instruments: Vec<String>,
}

#[derive(Args, Debug)]
struct TardisIngestorArgs {
    /// Venue name
    #[arg(long)]
    venue: String,

    /// Channel name
    #[arg(long)]
    channel: String,

    /// Instruments (comma-separated)
    #[arg(long, value_delimiter = ',', value_parser)]
    instruments: Vec<String>,

    /// Start datetime in "YYYY-MM-DD HH:MM" format
    #[arg(long, value_parser = parse_datetime)]
    start: OffsetDateTime,

    /// End datetime in "YYYY-MM-DD HH:MM" format
    #[arg(long, value_parser = parse_datetime)]
    end: OffsetDateTime,
}

#[derive(Args, Debug)]
struct EngineArgs {
    /// Instruments (comma-separated)
    #[arg(long, value_delimiter = ',', value_parser)]
    instruments: Vec<String>,
}

/// Custom parser to convert string to OffsetDateTime
fn parse_datetime(s: &str) -> Result<OffsetDateTime, String> {
    let format = format_description!("[year]-[month]-[day] [hour]:[minute]");
    let ts = PrimitiveDateTime::parse(&s, &format)
        .map_err(|e| format!("Failed to parse datetime '{}': {}", s, e))?
        .assume_utc();
    Ok(ts)
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();

    // Install the default CryptoProvider
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");

    let cli = Cli::parse();

    match cli.command {
        Commands::Insights(args) => {
            info!("Starting Arkin Pipeline 🚀");
            let res = run_insights(args).await;
            match res {
                Ok(_) => info!("Insights completed successfully"),
                Err(e) => error!("Insights failed: {}", e),
            }
        }
        Commands::Ingestors(args) => {
            info!("Starting Arkin Ingestors 🚀");
            let res = run_ingestor(args).await;
            match res {
                Ok(_) => info!("Ingestors completed successfully"),
                Err(e) => error!("Ingestors failed: {}", e),
            }
        }
        Commands::Engine(args) => {
            info!("Starting Arkin Trading Engine 🚀");
            let res = run_engine(args).await;
            match res {
                Ok(_) => info!("Engine completed successfully"),
                Err(e) => error!("Engine failed: {}", e),
            }
        }
    }
}

async fn run_insights(args: InsightsArgs) -> Result<()> {
    let mut instruments = vec![];
    let start = args.from;
    let end = args.till;

    let pubsub = Arc::new(PubSub::new());

    let config = load::<PersistenceConfig>();
    let persistence = Arc::new(PersistenceService::from_config(&config, pubsub.clone()).await);

    // Start the persistence service
    let persistence_task_tracker = TaskTracker::new();
    let persistence_shutdown = CancellationToken::new();
    let shutdown = persistence_shutdown.clone();
    let persistence_service = persistence.clone();
    persistence_task_tracker.spawn(async move {
        if let Err(e) = persistence_service.start(shutdown).await {
            error!("Failed to start persistence service: {}", e);
        }
    });

    let config = load::<InsightsConfig>().insights_service;
    let insights_service = Arc::new(InsightsService::from_config(&config, pubsub.clone(), persistence.clone()).await);

    // Fetch instruments concurrently
    for symbol in &args.instruments {
        match persistence.instrument_store.read_by_venue_symbol(symbol).await {
            Ok(instr) => instruments.push(instr),
            Err(e) => error!("Failed to read instrument {}: {}", symbol, e),
        }
    }

    info!("Loaded {} instruments.", instruments.len());

    let mut current_day = start.date();
    let frequency_secs = Duration::from_secs(config.frequency_secs);

    let mut clock = Clock::new(start, end, frequency_secs);

    while let Some((_tick_start, tick_end)) = clock.next() {
        if tick_end.date() != current_day {
            current_day = tick_end.date();

            // Remove the data
            insights_service.remove(tick_end).await?;

            // Load the data
            let tomorrow = tick_end + Duration::from_secs(86400);
            insights_service
                .load(tomorrow, &instruments, Duration::from_secs(86400))
                .await?;
        }
        insights_service.process(tick_end, &instruments, true).await?;
    }

    persistence.flush().await?;

    persistence_shutdown.cancel();
    persistence_task_tracker.close();
    persistence_task_tracker.wait().await;
    info!("Persistence service has shut down");
    Ok(())
}

async fn run_ingestor(args: IngestorsCommands) -> Result<()> {
    info!("Args: {:?}", args);
    let pubsub = Arc::new(PubSub::new());

    let config = load::<PersistenceConfig>();
    let persistence_service = Arc::new(PersistenceService::from_config(&config, pubsub.clone()).await);

    let config = load::<IngestorsConfig>();
    let ingestors = IngestorFactory::from_config(&config, pubsub.clone(), persistence_service.clone());

    // Start the persistence service
    let persistence_task_tracker = TaskTracker::new();
    let persistence_shutdown = CancellationToken::new();
    let shutdown = persistence_shutdown.clone();
    persistence_task_tracker.spawn(async move {
        if let Err(e) = persistence_service.start(shutdown).await {
            error!("Failed to start persistence service: {}", e);
        }
    });
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Start the ingestors
    let ingestor_task_tracker = TaskTracker::new();
    let ingestor_shutdown = CancellationToken::new();
    for ingestor in ingestors {
        let shutdown = ingestor_shutdown.clone();
        ingestor_task_tracker.spawn(async move {
            if let Err(e) = ingestor.start(shutdown).await {
                error!("Failed to start ingestor: {}", e);
            }
        });
    }

    match tokio::signal::ctrl_c().await {
        Ok(_) => {
            info!("Received Ctrl-C signal, shutting down...");
        }
        Err(e) => error!("Failed to listen for Ctrl-C signal: {}", e),
    }

    ingestor_shutdown.cancel();
    ingestor_task_tracker.close();
    ingestor_task_tracker.wait().await;
    info!("All ingestors have shut down");

    persistence_shutdown.cancel();
    persistence_task_tracker.close();
    persistence_task_tracker.wait().await;
    info!("Persistence service has shut down");
    Ok(())
}

async fn run_engine(args: EngineArgs) -> Result<()> {
    let pubsub = Arc::new(PubSub::new());
    info!("PubSub created");

    let config = load::<PersistenceConfig>();
    let persistence = Arc::new(PersistenceService::from_config(&config, pubsub.clone()).await);
    info!("Persistence created");

    let config = load::<PortfolioConfig>();
    let portfolio = PortfolioFactory::from_config(&config, pubsub.clone());
    info!("Portfolio created");

    let config = load::<IngestorsConfig>();
    let ingestors = IngestorFactory::from_config(&config, pubsub.clone(), persistence.clone());
    info!("Ingestors created");

    let config = load::<InsightsConfig>();
    let insights =
        Arc::new(InsightsService::from_config(&config.insights_service, pubsub.clone(), persistence.clone()).await);
    info!("Insights created");

    let config = load::<AllocationOptimConfig>();
    let allocation = AllocationFactory::from_config(&config, pubsub.clone(), persistence.clone(), portfolio.clone());
    info!("Allocation created");

    let config = load::<OrderManagerConfig>();
    let order_manager = ExecutionFactory::from_config(&config, pubsub.clone());
    info!("Order Manager created");

    let config = load::<ExecutorConfig>();
    let executor = ExecutorFactory::from_config(&config, pubsub.clone(), persistence.clone());
    info!("Executor created");

    // Work around for fetching instruments
    let mut instruments = vec![];
    for symbol in &args.instruments {
        match persistence.instrument_store.read_by_venue_symbol(symbol).await {
            Ok(instr) => instruments.push(instr),
            Err(e) => error!("Failed to read instrument {}: {}", symbol, e),
        }
    }
    info!("Loaded {} instruments.", instruments.len());

    let engine = ForecastEngine::builder()
        .pubsub(pubsub)
        .instruments(instruments)
        .persistor(persistence)
        .portfolio(portfolio)
        .ingestors(ingestors)
        .insights(insights)
        .allocation_optim(allocation)
        .order_manager(order_manager)
        .executor(executor)
        .build();

    engine.start().await.expect("Failed to start engine");

    info!("Waiting for shutdown to complete...");
    engine.stop().await.expect("Failed to stop engine");
    info!("Shutdown complete");
    Ok(())
}
