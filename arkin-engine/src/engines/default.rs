use std::sync::Arc;
use std::time::Instant;

use clap::Parser;
use tokio::signal::unix::{signal, SignalKind};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;

use crate::cli::{Cli, Commands, DownloadArgs, IngestorsArgs, InsightsArgs, SimulationArgs};
use crate::config::EngineConfig;
use crate::factories::{
    AccountingFactory, AllocationFactory, ExecutorFactory, IngestorFactory, InsightsFactory, OrderManagerFactory,
    StrategyFactory,
};
use crate::TradingEngineError;

pub struct DefaultEngine {
    timer: Instant,
    pubsub: Arc<PubSub>,
    persistence: Arc<PersistenceService>,
    service_tracker: TaskTracker,
    service_shutdown: CancellationToken,

    core_service_shutdown: CancellationToken,
    core_service_tracker: TaskTracker,
}

impl DefaultEngine {
    pub async fn new() -> Self {
        let config = load::<EngineConfig>();
        let args = Cli::parse();

        // Check if default engine is configured
        let config = config.engine.default.expect("Default engine not configured");
        let pubsub = PubSub::new(config.pubsub_capacity);

        // Init persistence
        let config = load::<PersistenceConfig>();
        let instance = match &args.command {
            Commands::Simulation(args) => Instance::builder()
                .name(args.instance_name.clone())
                .instance_type(InstanceType::Simulation)
                .build(),
            Commands::Live(args) => Instance::builder()
                .name(args.instance_name.clone())
                .instance_type(InstanceType::Live)
                .build(),
            _ => Instance::builder()
                .name("other".to_string())
                .instance_type(InstanceType::Utility)
                .build(),
        };

        let only_predictions = match &args.command {
            Commands::Download(_args) => false,
            Commands::Ingestor(_args) => false,
            Commands::Insights(args) => args.only_predictions,
            Commands::Simulation(_args) => false,
            Commands::Live(_args) => false,
        };

        let only_normalized = match &args.command {
            Commands::Download(_args) => false,
            Commands::Ingestor(_args) => false,
            Commands::Insights(args) => args.only_normalized,
            Commands::Simulation(_args) => false,
            Commands::Live(_args) => false,
        };

        let dry_run = match &args.command {
            Commands::Download(args) => args.dry_run,
            Commands::Ingestor(args) => args.dry_run,
            Commands::Insights(args) => args.dry_run,
            Commands::Simulation(args) => args.dry_run,
            Commands::Live(_args) => false,
        };
        let persistence =
            PersistenceService::new(pubsub.clone(), &config, instance, only_normalized, only_predictions, dry_run)
                .await;

        let engine = Self {
            timer: Instant::now(),
            pubsub,
            persistence: persistence.clone(),
            service_tracker: TaskTracker::new(),
            service_shutdown: CancellationToken::new(),
            core_service_shutdown: CancellationToken::new(),
            core_service_tracker: TaskTracker::new(),
        };

        engine.start_service(persistence, true).await;

        let res = match &args.command {
            Commands::Download(args) => engine.run_download(&args).await,
            Commands::Ingestor(args) => engine.run_ingestor(&args).await,
            Commands::Insights(args) => engine.run_insights(&args).await,
            Commands::Simulation(args) => engine.run_simulation(&args).await,
            Commands::Live(_args) => unimplemented!(),
        };
        match res {
            Ok(_) => info!("Engine started successfully"),
            Err(e) => {
                error!("Failed to start engine: {:?}", e);
                engine.shutdown().await;
            }
        }
        engine
    }

    pub async fn run_download(&self, args: &DownloadArgs) -> Result<(), TradingEngineError> {
        let ingestor = IngestorFactory::init_download(self.pubsub.clone(), self.persistence.clone(), &args);
        self.start_service(ingestor, false).await;
        Ok(())
    }

    pub async fn run_ingestor(&self, args: &IngestorsArgs) -> Result<(), TradingEngineError> {
        let ingestor = IngestorFactory::init(self.pubsub.clone(), self.persistence.clone(), args);
        self.start_service(ingestor, false).await;
        Ok(())
    }

    pub async fn run_insights(&self, args: &InsightsArgs) -> Result<(), TradingEngineError> {
        let insights = InsightsFactory::init(self.pubsub.clone(), self.persistence.clone(), &args.pipeline).await;
        let ingestor = IngestorFactory::init_insights(self.pubsub.clone(), self.persistence.clone(), args).await;

        self.start_service(insights, false).await;
        self.start_service(ingestor, false).await;
        Ok(())
    }

    pub async fn run_simulation(&self, args: &SimulationArgs) -> Result<(), TradingEngineError> {
        info!("Running simulation...");
        // Init services
        info!("Initializing services...");
        let insights = InsightsFactory::init(self.pubsub.clone(), self.persistence.clone(), &args.pipeline).await;
        info!("Insights initialized");
        let ingestor = IngestorFactory::init_simulation(self.pubsub.clone(), self.persistence.clone(), args).await;
        let accounting =
            AccountingFactory::init(self.pubsub.clone(), self.persistence.clone(), &args.accounting_type).await;
        info!("Accounting initialized");
        let strategies = StrategyFactory::init(self.pubsub.clone(), self.persistence.clone()).await;
        info!("Strategies initialized");
        let allocation = AllocationFactory::init(self.pubsub.clone(), self.persistence.clone(), accounting.clone());
        info!("Allocation initialized");
        let order_manager = OrderManagerFactory::init(self.pubsub.clone());
        info!("Order Manager initialized");
        let execution = ExecutorFactory::init_simulation(self.pubsub.clone());
        info!("Execution initialized");
        info!("strategy count: {}", strategies.len());

        let mut services: Vec<Arc<dyn RunnableService>> =
            vec![insights, ingestor, accounting, allocation, order_manager, execution];
        for strategy in strategies {
            services.push(strategy);
        }

        self.start_services(services, false).await;
        Ok(())
    }

    async fn start_service(&self, service: Arc<dyn RunnableService>, core_service: bool) {
        match core_service {
            true => {
                let shutdown = self.core_service_shutdown.clone();
                self.core_service_tracker.spawn(async move {
                    if let Err(e) = service.start(shutdown).await {
                        error!("Core service error: {:?}", e);
                    }
                });
            }
            false => {
                let shutdown = self.service_shutdown.clone();
                self.service_tracker.spawn(async move {
                    if let Err(e) = service.start(shutdown).await {
                        error!("Service error: {:?}", e);
                    }
                });
            }
        }
    }

    async fn start_services<I>(&self, services: I, core_service: bool)
    where
        I: IntoIterator<Item = Arc<dyn RunnableService>>,
    {
        for service in services {
            self.start_service(service, core_service).await;
        }
    }

    pub async fn wait_for_shutdown(&self) {
        let mut sigterm = signal(SignalKind::terminate()).unwrap();
        let mut sigint = signal(SignalKind::interrupt()).unwrap();
        let mut rx = self.pubsub.subscribe();
        loop {
            tokio::select! {
                _ = sigterm.recv() => {
                  info!("Received terminate signal, shutting down...");
                  self.shutdown().await;
                  break;
                },
                _ = sigint.recv() => {
                  info!("Received interrupt signal, shutting down...");
                  self.shutdown().await;
                  break;
                },
                event = rx.recv() => {
                    if let Ok(Event::Finished) = event {
                      info!("Received finished event, shutting down...");
                        self.shutdown().await;
                        break;
                    }
                }
            }
        }
        info!("Successfully shutdown!");
    }

    pub async fn shutdown(&self) {
        info!("Shutting down services...");
        self.service_shutdown.cancel();
        self.service_tracker.close();
        self.service_tracker.wait().await;

        info!("Shutting down core services...");
        self.core_service_shutdown.cancel();
        self.core_service_tracker.close();
        self.core_service_tracker.wait().await;

        info!("Service was running for {:?}", self.timer.elapsed());
    }
}
