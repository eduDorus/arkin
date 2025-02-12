use std::{sync::Arc, time::Duration};

use tokio::signal::unix::{signal, SignalKind};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

use arkin_allocation::prelude::*;
use arkin_core::prelude::*;
use arkin_executors::prelude::*;
use arkin_ingestors::prelude::*;
use arkin_insights::prelude::*;
use arkin_ordermanager::prelude::*;
use arkin_persistence::prelude::*;
use arkin_portfolio::prelude::*;

use crate::TradingEngineError;

pub struct DefaultEngine {
    pubsub: Arc<PubSub>,
    persistence: Arc<PersistenceService>,
    service_tracker: TaskTracker,
    service_shutdown: CancellationToken,

    core_service_shutdown: CancellationToken,
    core_service_tracker: TaskTracker,
}

impl DefaultEngine {
    pub async fn new() -> Self {
        // Init pubsub
        let pubsub = PubSub::new(1000000);

        // Init persistence
        let config = load::<PersistenceConfig>();
        let persistence = PersistenceService::new(pubsub.clone(), &config, true).await;

        let engine = Self {
            pubsub,
            persistence: persistence.clone(),
            service_tracker: TaskTracker::new(),
            service_shutdown: CancellationToken::new(),
            core_service_shutdown: CancellationToken::new(),
            core_service_tracker: TaskTracker::new(),
        };

        engine.start_service(persistence, true).await;
        engine
    }

    pub async fn run_ingestor(&self, args: &IngestorsCommands) -> Result<(), TradingEngineError> {
        let config = load::<IngestorsConfig>();
        let ingestor = IngestorFactory::init(self.pubsub.clone(), self.persistence.clone(), &config.ingestors, args)?;
        self.start_service(ingestor, false).await;
        Ok(())
    }

    pub async fn run_insights(&self, args: &InsightsArgs) -> Result<(), TradingEngineError> {
        // Load Instruments
        let mut instruments = vec![];
        for symbol in &args.instruments {
            match self.persistence.instrument_store.read_by_venue_symbol(symbol).await {
                Ok(instr) => instruments.push(instr),
                Err(e) => error!("Failed to read instrument {}: {}", symbol, e),
            }
        }

        // Load Pipeline
        let pipeline = self.persistence.pipeline_store.read_by_name(&args.pipeline).await?;
        let insights = InsightsService::init(self.pubsub.clone(), pipeline).await;

        // Load Simulation ingestor
        let ingestor = IngestorFactory::init_simulation(
            self.pubsub.clone(),
            self.persistence.clone(),
            instruments,
            Duration::from_secs(60),
            args.start,
            args.end,
        );

        self.start_service(insights, false).await;
        self.start_service(ingestor, false).await;

        Ok(())
    }

    pub async fn run_simulation(&self, args: &SimulationArgs) -> Result<(), TradingEngineError> {
        // Load Instruments
        let mut instruments = vec![];
        for symbol in &args.instruments {
            match self.persistence.instrument_store.read_by_venue_symbol(symbol).await {
                Ok(instr) => instruments.push(instr),
                Err(e) => error!("Failed to read instrument {}: {}", symbol, e),
            }
        }

        // Load Pipeline
        let pipeline = self.persistence.pipeline_store.read_by_name(&args.pipeline).await?;
        let insights = InsightsService::init(self.pubsub.clone(), pipeline).await;

        // Load Simulation ingestor
        let ingestor = IngestorFactory::init_simulation(
            self.pubsub.clone(),
            self.persistence.clone(),
            instruments,
            Duration::from_secs(60),
            args.start,
            args.end,
        );

        // Load strategies
        let portfolio = PortfolioFactory::init(self.pubsub.clone());
        let allocation = AllocationFactory::init(self.pubsub.clone(), self.persistence.clone(), portfolio.clone());
        let order_manager = OrderManagerFactory::init(self.pubsub.clone());
        let execution = ExecutorFactory::init(self.pubsub.clone(), self.persistence.clone());

        self.start_service(execution, false).await;
        self.start_service(order_manager, false).await;
        self.start_service(portfolio, false).await;
        self.start_service(allocation, false).await;
        self.start_service(insights, false).await;
        self.start_service(ingestor, false).await;

        Ok(())
    }

    async fn start_service(&self, service: Arc<dyn RunnableService>, core_service: bool) {
        match core_service {
            true => {
                let shutdown = self.core_service_shutdown.clone();
                self.core_service_tracker.spawn(async move {
                    let res = service.start(shutdown).await;
                    match res {
                        Ok(_) => info!("Service finished"),
                        Err(e) => error!("Service error: {:?}", e),
                    }
                });
            }
            false => {
                let shutdown = self.service_shutdown.clone();
                self.service_tracker.spawn(async move {
                    let res = service.start(shutdown).await;
                    match res {
                        Ok(_) => info!("Service finished"),
                        Err(e) => error!("Service error: {:?}", e),
                    }
                });
            }
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
    }
}
