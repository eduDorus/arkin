use std::{sync::Arc, time::Duration};

use anyhow::Result;
use tokio_rustls::rustls::crypto::{aws_lc_rs, CryptoProvider};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

use arkin_cli::prelude::*;
use arkin_core::prelude::*;
use arkin_ingestors::prelude::*;
use arkin_insights::prelude::*;
use arkin_persistence::prelude::*;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();

    // Install the default CryptoProvider
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");

    let cli = parse_cli();

    let res = match cli.command {
        Commands::Ingestors(args) => {
            info!("Starting arkin Ingestors 🚀");
            run_ingestor(args).await
        }
        Commands::Insights(args) => {
            info!("Starting arkin Pipeline 🚀");
            run_insights(args).await
        }
        Commands::Simulation(args) => {
            info!("Starting arkin Simulation 🚀");
            run_simulation(args).await
        }
        Commands::Engine(args) => {
            info!("Starting arkin Trading Engine 🚀");
            run_engine(args).await
        }
    };

    match res {
        Ok(_) => info!("Completed successfully!"),
        Err(e) => error!("Failed: {}", e),
    }
}

async fn run_insights(args: InsightsArgs) -> Result<()> {
    let mut instruments = vec![];
    let start = args.from;
    let end = args.till;

    let pubsub = Arc::new(PubSub::new(10240));
    let persistence = PersistenceFactory::init_from_config(pubsub.clone(), args.dry_run).await;

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

    // while let Some((_tick_start, tick_end)) = clock.next() {
    //     if tick_end.date() != current_day {
    //         current_day = tick_end.date();

    //         // Remove the data
    //         insights_service.remove(tick_end).await?;

    //         // Load the data
    //         let tomorrow = tick_end + Duration::from_secs(86400);
    //         let timer = Instant::now();
    //         // insights_service
    //         //     .load(tomorrow, &instruments, Duration::from_secs(86400))
    //         //     .await?;
    //         debug!("Loaded insights in {:?}", timer.elapsed());
    //     }
    //     let timer = Instant::now();
    //     insights_service.process(tick_end, &instruments, true).await?;
    //     debug!("Processed insights in {:?}", timer.elapsed());
    // }

    persistence.flush().await?;
    persistence_shutdown.cancel();
    persistence_task_tracker.close();
    persistence_task_tracker.wait().await;
    Ok(())
}

async fn run_ingestor(args: IngestorsCommands) -> Result<()> {
    info!("Args: {:?}", args);

    let pubsub = Arc::new(PubSub::new(1000000));
    let persistence_service = PersistenceFactory::init_from_config(pubsub.clone(), args.dry_run).await;

    let ingestor_task_tracker = TaskTracker::new();
    let ingestor_shutdown = CancellationToken::new();
    let shutdown = ingestor_shutdown.clone();

    match args {
        IngestorsCommands::Binance(args) => {
            let ingestor = IngestorFactory::create_binance_ingestor(
                pubsub.clone(),
                persistence_service.clone(),
                args.channels,
                args.instruments,
            );
            ingestor_task_tracker.spawn(async move {
                let res = ingestor.start(shutdown).await;
                match res {
                    Ok(_) => info!("Ingestor completed successfully"),
                    Err(e) => error!("Ingestor failed: {}", e),
                }
            });

            match tokio::signal::ctrl_c().await {
                Ok(_) => {
                    info!("Received Ctrl-C signal, shutting down...");
                }
                Err(e) => error!("Failed to listen for Ctrl-C signal: {}", e),
            }

            ingestor_shutdown.cancel();
            ingestor_task_tracker.close();
            ingestor_task_tracker.wait().await;
        }
        IngestorsCommands::Tardis(args) => {
            let ingestor = IngestorFactory::create_tardis_ingestor(
                pubsub.clone(),
                persistence_service.clone(),
                args.venue,
                args.channel,
                args.instruments,
                args.start,
                args.end,
            );
            let ingestor_task_tracker = TaskTracker::new();
            let ingestor_shutdown = CancellationToken::new();
            let shutdown = ingestor_shutdown.clone();
            ingestor_task_tracker.spawn(async move {
                let res = ingestor.start(shutdown).await;
                match res {
                    Ok(_) => info!("Ingestor completed successfully"),
                    Err(e) => error!("Ingestor failed: {}", e),
                }
            });

            match tokio::signal::ctrl_c().await {
                Ok(_) => {
                    info!("Received Ctrl-C signal, shutting down...");
                }
                Err(e) => error!("Failed to listen for Ctrl-C signal: {}", e),
            }

            ingestor_shutdown.cancel();
            ingestor_task_tracker.close();
            ingestor_task_tracker.wait().await;
        }
    }
    // let config = load::<IngestorsConfig>();
    // let ingestors = IngestorFactory::from_config(&config, pubsub.clone(), persistence_service.clone());

    // // Start the persistence service
    // let persistence_task_tracker = TaskTracker::new();
    // let persistence_shutdown = CancellationToken::new();
    // let shutdown = persistence_shutdown.clone();
    // persistence_task_tracker.spawn(async move {
    //     if let Err(e) = persistence_service.start(shutdown).await {
    //         error!("Failed to start persistence service: {}", e);
    //     }
    // });
    // tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // // Start the ingestors
    // let ingestor_task_tracker = TaskTracker::new();
    // let ingestor_shutdown = CancellationToken::new();
    // for ingestor in ingestors {
    //     let shutdown = ingestor_shutdown.clone();
    //     ingestor_task_tracker.spawn(async move {
    //         if let Err(e) = ingestor.start(shutdown).await {
    //             error!("Failed to start ingestor: {}", e);
    //         }
    //     });
    // }

    // match tokio::signal::ctrl_c().await {
    //     Ok(_) => {
    //         info!("Received Ctrl-C signal, shutting down...");
    //     }
    //     Err(e) => error!("Failed to listen for Ctrl-C signal: {}", e),
    // }

    // ingestor_shutdown.cancel();
    // ingestor_task_tracker.close();
    // ingestor_task_tracker.wait().await;

    // persistence_shutdown.cancel();
    // persistence_task_tracker.close();
    // persistence_task_tracker.wait().await;
    Ok(())
}

async fn run_simulation(args: SimulationArgs) -> Result<()> {
    let pubsub = Arc::new(PubSub::new(10000000));

    let persistence = PersistenceFactory::init_from_config(pubsub.clone(), args.dry_run).await;

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

    // Start the insights service
    let insights_task_tracker = TaskTracker::new();
    let insights_shutdown = CancellationToken::new();
    let shutdown = insights_shutdown.clone();
    insights_task_tracker.spawn(async move {
        if let Err(e) = insights_service.start(shutdown).await {
            error!("Failed to start insights service: {}", e);
        }
    });

    // Fetch instruments concurrently
    let mut instruments = vec![];
    for symbol in &args.instruments {
        match persistence.instrument_store.read_by_venue_symbol(symbol).await {
            Ok(instr) => instruments.push(instr),
            Err(e) => error!("Failed to read instrument {}: {}", symbol, e),
        }
    }
    info!("Loaded {} instruments.", instruments.len());

    // Create ingestor
    let ingestor = IngestorFactory::create_simulation_ingestor(
        pubsub.clone(),
        persistence.clone(),
        instruments,
        Duration::from_secs(config.frequency_secs),
        args.start,
        args.end,
    );

    let ingestor_task_tracker = TaskTracker::new();
    let ingestor_shutdown = CancellationToken::new();
    let shutdown = ingestor_shutdown.clone();
    ingestor_task_tracker.spawn(async move {
        let res = ingestor.start(shutdown).await;
        match res {
            Ok(_) => info!("Ingestor completed successfully"),
            Err(e) => error!("Simulation failed: {}", e),
        }
    });

    let mut rx = pubsub.subscribe();
    info!("Waiting for pubsub messages...");
    // Consume the pubsub messages in case of error log it
    let mut trade_counter = 0;
    let mut tick_counter = 0;
    while let Ok(event) = rx.recv().await {
        match event {
            Event::Tick(_tick) => {
                tick_counter += 1;
            }
            Event::Trade(_trade) => {
                trade_counter += 1;
            }
            Event::Finished => {
                break;
            }
            _ => {}
        }
    }
    info!("Received {} ticks and {} trades", tick_counter, trade_counter);

    ingestor_shutdown.cancel();
    ingestor_task_tracker.close();
    ingestor_task_tracker.wait().await;

    insights_shutdown.cancel();
    insights_task_tracker.close();
    insights_task_tracker.wait().await;

    persistence.flush().await?;
    persistence_shutdown.cancel();
    persistence_task_tracker.close();
    persistence_task_tracker.wait().await;

    Ok(())
}

async fn run_engine(_args: EngineArgs) -> Result<()> {
    // let pubsub = Arc::new(PubSub::new(1000000));
    // info!("PubSub created");

    // let config = load::<PersistenceConfig>();
    // let persistence = Arc::new(PersistenceService::from_config(&config, pubsub.clone()).await);
    // info!("Persistence created");

    // let config = load::<PortfolioConfig>();
    // let portfolio = PortfolioFactory::from_config(&config, pubsub.clone());
    // info!("Portfolio created");

    // let config = load::<IngestorsConfig>();
    // let ingestors = IngestorFactory::from_config(&config, pubsub.clone(), persistence.clone());
    // info!("Ingestors created");

    // let config = load::<InsightsConfig>();
    // let insights =
    //     Arc::new(InsightsService::from_config(&config.insights_service, pubsub.clone(), persistence.clone()).await);
    // info!("Insights created");

    // let config = load::<AllocationOptimConfig>();
    // let allocation = AllocationFactory::from_config(&config, pubsub.clone(), persistence.clone(), portfolio.clone());
    // info!("Allocation created");

    // let config = load::<OrderManagerConfig>();
    // let order_manager = ExecutionFactory::from_config(&config, pubsub.clone());
    // info!("Order Manager created");

    // let config = load::<ExecutorConfig>();
    // let executor = ExecutorFactory::from_config(&config, pubsub.clone(), persistence.clone());
    // info!("Executor created");

    // // Work around for fetching instruments
    // let mut instruments = vec![];
    // for symbol in &args.instruments {
    //     match persistence.instrument_store.read_by_venue_symbol(symbol).await {
    //         Ok(instr) => instruments.push(instr),
    //         Err(e) => error!("Failed to read instrument {}: {}", symbol, e),
    //     }
    // }
    // info!("Loaded {} instruments.", instruments.len());

    // let engine = ForecastEngine::builder()
    //     .pubsub(pubsub)
    //     .instruments(instruments)
    //     .persistor(persistence)
    //     .portfolio(portfolio)
    //     .ingestors(ingestors)
    //     .insights(insights)
    //     .allocation_optim(allocation)
    //     .order_manager(order_manager)
    //     .executor(executor)
    //     .build();

    // engine.start().await.expect("Failed to start engine");

    // info!("Waiting for shutdown to complete...");
    // engine.stop().await.expect("Failed to stop engine");
    // info!("Shutdown complete");
    Ok(())
}
