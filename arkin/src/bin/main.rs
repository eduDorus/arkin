use clap::Parser;
use tokio_rustls::rustls::crypto::{aws_lc_rs, CryptoProvider};
use tracing::info;

use arkin_core::prelude::*;
use arkin_engine::prelude::*;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();

    // Install the default CryptoProvider
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");

    let cli = Cli::parse();

    match cli.command {
        Commands::Ingestor(args) => {
            info!("Starting arkin Ingestor 🚀");
            info!("Args: {:?}", args);
            let engine = DefaultEngine::new().await;
            engine.run_ingestor(&args).await.expect("Failed to start ingestor");
            engine.wait_for_shutdown().await;
        }
        Commands::Insights(args) => {
            info!("Starting arkin Pipeline 🚀");
            info!("Args: {:?}", args);
            let engine = DefaultEngine::new().await;
            engine.run_insights(&args).await.expect("Failed to start insights");
            engine.wait_for_shutdown().await;
        }
        Commands::Simulation(args) => {
            info!("Starting arkin Simulation 🚀");
            info!("Args: {:?}", args);
            let engine = DefaultEngine::new().await;
            engine.run_simulation(&args).await.expect("Failed to start simulation");
            engine.wait_for_shutdown().await;
        }
        Commands::Live(args) => {
            info!("Starting arkin Trading Engine 🚀");
            info!("Args: {:?}", args);
        }
    };
}

// async fn run_insights(args: InsightsArgs) -> Result<()> {
//     let start = args.from;
//     let end = args.till;
//     let dry_run = args.dry_run;

//     // 1. Create pubsub
//     let pubsub = PubSub::new(1_000_000);

//     // 2. Create persistence
//     let config = load::<PersistenceConfig>();
//     let persistence = PersistenceService::new(pubsub.clone(), &config, dry_run).await;

//     // 3. Create insights service
//     let config = load::<InsightsConfig>().insights_service;
//     let pipeline = persistence.pipeline_store.read_by_name(&config.pipeline.name).await?;
//     let insights = InsightsService::init(pubsub.clone(), &config, pipeline).await;

//     // 4. Create ingestor
//     let mut instruments = vec![];
//     for symbol in &args.instruments {
//         match persistence.instrument_store.read_by_venue_symbol(symbol).await {
//             Ok(instr) => instruments.push(instr),
//             Err(e) => error!("Failed to read instrument {}: {}", symbol, e),
//         }
//     }
//     info!("Loaded {} instruments.", instruments.len());

//     // Create ingestor
//     let ingestor = IngestorFactory::create_simulation_ingestor(
//         pubsub.clone(),
//         persistence.clone(),
//         instruments,
//         Duration::from_secs(config.frequency_secs),
//         start,
//         end,
//     );

//     // Start the persistence service
//     let persistence_task_tracker = TaskTracker::new();
//     let persistence_shutdown = CancellationToken::new();
//     let shutdown = persistence_shutdown.clone();
//     let persistence_service = persistence.clone();
//     persistence_task_tracker.spawn(async move {
//         if let Err(e) = persistence_service.start(shutdown).await {
//             error!("Failed to start persistence service: {}", e);
//         }
//     });

//     // Start the insights service
//     let insights_task_tracker = TaskTracker::new();
//     let insights_shutdown = CancellationToken::new();
//     let shutdown = insights_shutdown.clone();
//     insights_task_tracker.spawn(async move {
//         if let Err(e) = insights.start(shutdown).await {
//             error!("Failed to start insights service: {}", e);
//         }
//     });

//     // Start the ingestor
//     let ingestor_task_tracker = TaskTracker::new();
//     let ingestor_shutdown = CancellationToken::new();
//     let shutdown = ingestor_shutdown.clone();
//     ingestor_task_tracker.spawn(async move {
//         let res = ingestor.start(shutdown).await;
//         match res {
//             Ok(_) => info!("Ingestor completed successfully"),
//             Err(e) => error!("Ingestor failed: {}", e),
//         }
//     });

//     // Wait for the ingestor to finish
//     let mut rx = pubsub.subscribe();
//     while let Ok(event) = rx.recv().await {
//         match event {
//             Event::Finished => {
//                 break;
//             }
//             _ => {}
//         }
//     }
//     ingestor_shutdown.cancel();
//     ingestor_task_tracker.close();
//     ingestor_task_tracker.wait().await;

//     insights_shutdown.cancel();
//     insights_task_tracker.close();
//     insights_task_tracker.wait().await;

//     persistence.flush().await?;
//     persistence_shutdown.cancel();
//     persistence_task_tracker.close();
//     persistence_task_tracker.wait().await;

//     Ok(())
// }

// async fn run_ingestor(args: IngestorsCommands) -> Result<()> {
//     info!("Args: {:?}", args);

//     let pubsub = Arc::new(PubSub::new(1000000));
//     let persistence_service = PersistenceFactory::init(pubsub.clone(), args.dry_run).await;

//     let ingestor_task_tracker = TaskTracker::new();
//     let ingestor_shutdown = CancellationToken::new();
//     let shutdown = ingestor_shutdown.clone();

//     match args {
//         IngestorsCommands::Binance(args) => {
//             let ingestor = IngestorFactory::create_binance_ingestor(
//                 pubsub.clone(),
//                 persistence_service.clone(),
//                 args.channels,
//                 args.instruments,
//             );
//             ingestor_task_tracker.spawn(async move {
//                 let res = ingestor.start(shutdown).await;
//                 match res {
//                     Ok(_) => info!("Ingestor completed successfully"),
//                     Err(e) => error!("Ingestor failed: {}", e),
//                 }
//             });

//             match tokio::signal::ctrl_c().await {
//                 Ok(_) => {
//                     info!("Received Ctrl-C signal, shutting down...");
//                 }
//                 Err(e) => error!("Failed to listen for Ctrl-C signal: {}", e),
//             }

//             ingestor_shutdown.cancel();
//             ingestor_task_tracker.close();
//             ingestor_task_tracker.wait().await;
//         }
//         IngestorsCommands::Tardis(args) => {
//             let ingestor = IngestorFactory::create_tardis_ingestor(
//                 pubsub.clone(),
//                 persistence_service.clone(),
//                 args.venue,
//                 args.channel,
//                 args.instruments,
//                 args.start,
//                 args.end,
//             );
//             let ingestor_task_tracker = TaskTracker::new();
//             let ingestor_shutdown = CancellationToken::new();
//             let shutdown = ingestor_shutdown.clone();
//             ingestor_task_tracker.spawn(async move {
//                 let res = ingestor.start(shutdown).await;
//                 match res {
//                     Ok(_) => info!("Ingestor completed successfully"),
//                     Err(e) => error!("Ingestor failed: {}", e),
//                 }
//             });

//             match tokio::signal::ctrl_c().await {
//                 Ok(_) => {
//                     info!("Received Ctrl-C signal, shutting down...");
//                 }
//                 Err(e) => error!("Failed to listen for Ctrl-C signal: {}", e),
//             }

//             ingestor_shutdown.cancel();
//             ingestor_task_tracker.close();
//             ingestor_task_tracker.wait().await;
//         }
//     }
//     // let config = load::<IngestorsConfig>();
//     // let ingestors = IngestorFactory::from_config(&config, pubsub.clone(), persistence_service.clone());

//     // // Start the persistence service
//     // let persistence_task_tracker = TaskTracker::new();
//     // let persistence_shutdown = CancellationToken::new();
//     // let shutdown = persistence_shutdown.clone();
//     // persistence_task_tracker.spawn(async move {
//     //     if let Err(e) = persistence_service.start(shutdown).await {
//     //         error!("Failed to start persistence service: {}", e);
//     //     }
//     // });
//     // tokio::time::sleep(std::time::Duration::from_secs(1)).await;

//     // // Start the ingestors
//     // let ingestor_task_tracker = TaskTracker::new();
//     // let ingestor_shutdown = CancellationToken::new();
//     // for ingestor in ingestors {
//     //     let shutdown = ingestor_shutdown.clone();
//     //     ingestor_task_tracker.spawn(async move {
//     //         if let Err(e) = ingestor.start(shutdown).await {
//     //             error!("Failed to start ingestor: {}", e);
//     //         }
//     //     });
//     // }

//     // match tokio::signal::ctrl_c().await {
//     //     Ok(_) => {
//     //         info!("Received Ctrl-C signal, shutting down...");
//     //     }
//     //     Err(e) => error!("Failed to listen for Ctrl-C signal: {}", e),
//     // }

//     // ingestor_shutdown.cancel();
//     // ingestor_task_tracker.close();
//     // ingestor_task_tracker.wait().await;

//     // persistence_shutdown.cancel();
//     // persistence_task_tracker.close();
//     // persistence_task_tracker.wait().await;
//     Ok(())
// }

// async fn run_simulation(args: SimulationArgs) -> Result<()> {
//     let pubsub = Arc::new(PubSub::new(10000000));

//     let config = load::<PersistenceConfig>();
//     let persistence = PersistenceFactory::init(&config, pubsub.clone(), args.dry_run).await;

//     let config = load::<InsightsConfig>().insights_service;
//     let insights_service = Arc::new(InsightsService::init(&config, pubsub.clone()).await);

//     // Fetch instruments
//     let mut instruments = vec![];
//     for symbol in &args.instruments {
//         match persistence.instrument_store.read_by_venue_symbol(symbol).await {
//             Ok(instr) => instruments.push(instr),
//             Err(e) => error!("Failed to read instrument {}: {}", symbol, e),
//         }
//     }
//     info!("Loaded {} instruments.", instruments.len());

//     // Create ingestor
//     let ingestor = IngestorFactory::create_simulation_ingestor(
//         pubsub.clone(),
//         persistence.clone(),
//         instruments,
//         Duration::from_secs(config.frequency_secs),
//         args.start,
//         args.end,
//     );

//     // Start the persistence service
//     let persistence_task_tracker = TaskTracker::new();
//     let persistence_shutdown = CancellationToken::new();
//     let shutdown = persistence_shutdown.clone();
//     let persistence_service = persistence.clone();
//     persistence_task_tracker.spawn(async move {
//         if let Err(e) = persistence_service.start(shutdown).await {
//             error!("Failed to start persistence service: {}", e);
//         }
//     });

//     // Start the insights service
//     let insights_task_tracker = TaskTracker::new();
//     let insights_shutdown = CancellationToken::new();
//     let shutdown = insights_shutdown.clone();
//     insights_task_tracker.spawn(async move {
//         if let Err(e) = insights_service.start(shutdown).await {
//             error!("Failed to start insights service: {}", e);
//         }
//     });

//     let ingestor_task_tracker = TaskTracker::new();
//     let ingestor_shutdown = CancellationToken::new();
//     let shutdown = ingestor_shutdown.clone();
//     ingestor_task_tracker.spawn(async move {
//         let res = ingestor.start(shutdown).await;
//         match res {
//             Ok(_) => info!("Ingestor completed successfully"),
//             Err(e) => error!("Simulation failed: {}", e),
//         }
//     });

//     let mut rx = pubsub.subscribe();
//     info!("Waiting for pubsub messages...");
//     // Consume the pubsub messages in case of error log it
//     let mut trade_counter = 0;
//     let mut tick_counter = 0;
//     while let Ok(event) = rx.recv().await {
//         match event {
//             Event::Tick(_tick) => {
//                 tick_counter += 1;
//             }
//             Event::Trade(_trade) => {
//                 trade_counter += 1;
//             }
//             Event::Finished => {
//                 break;
//             }
//             _ => {}
//         }
//     }
//     info!("Received {} ticks and {} trades", tick_counter, trade_counter);

//     ingestor_shutdown.cancel();
//     ingestor_task_tracker.close();
//     ingestor_task_tracker.wait().await;

//     insights_shutdown.cancel();
//     insights_task_tracker.close();
//     insights_task_tracker.wait().await;

//     persistence.flush().await?;
//     persistence_shutdown.cancel();
//     persistence_task_tracker.close();
//     persistence_task_tracker.wait().await;

//     Ok(())
// }

// async fn run_engine(_args: EngineArgs) -> Result<()> {
//     // let pubsub = Arc::new(PubSub::new(1000000));
//     // info!("PubSub created");

//     // let config = load::<PersistenceConfig>();
//     // let persistence = Arc::new(PersistenceService::from_config(&config, pubsub.clone()).await);
//     // info!("Persistence created");

//     // let config = load::<PortfolioConfig>();
//     // let portfolio = PortfolioFactory::from_config(&config, pubsub.clone());
//     // info!("Portfolio created");

//     // let config = load::<IngestorsConfig>();
//     // let ingestors = IngestorFactory::from_config(&config, pubsub.clone(), persistence.clone());
//     // info!("Ingestors created");

//     // let config = load::<InsightsConfig>();
//     // let insights =
//     //     Arc::new(InsightsService::from_config(&config.insights_service, pubsub.clone(), persistence.clone()).await);
//     // info!("Insights created");

//     // let config = load::<AllocationOptimConfig>();
//     // let allocation = AllocationFactory::from_config(&config, pubsub.clone(), persistence.clone(), portfolio.clone());
//     // info!("Allocation created");

//     // let config = load::<OrderManagerConfig>();
//     // let order_manager = ExecutionFactory::from_config(&config, pubsub.clone());
//     // info!("Order Manager created");

//     // let config = load::<ExecutorConfig>();
//     // let executor = ExecutorFactory::from_config(&config, pubsub.clone(), persistence.clone());
//     // info!("Executor created");

//     // // Work around for fetching instruments
//     // let mut instruments = vec![];
//     // for symbol in &args.instruments {
//     //     match persistence.instrument_store.read_by_venue_symbol(symbol).await {
//     //         Ok(instr) => instruments.push(instr),
//     //         Err(e) => error!("Failed to read instrument {}: {}", symbol, e),
//     //     }
//     // }
//     // info!("Loaded {} instruments.", instruments.len());

//     // let engine = ForecastEngine::builder()
//     //     .pubsub(pubsub)
//     //     .instruments(instruments)
//     //     .persistor(persistence)
//     //     .portfolio(portfolio)
//     //     .ingestors(ingestors)
//     //     .insights(insights)
//     //     .allocation_optim(allocation)
//     //     .order_manager(order_manager)
//     //     .executor(executor)
//     //     .build();

//     // engine.start().await.expect("Failed to start engine");

//     // info!("Waiting for shutdown to complete...");
//     // engine.stop().await.expect("Failed to stop engine");
//     // info!("Shutdown complete");
//     Ok(())
// }
