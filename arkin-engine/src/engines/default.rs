use std::sync::Arc;

use tokio::signal::unix::{signal, SignalKind};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

use arkin_core::prelude::*;
use arkin_ingestors::prelude::*;
use arkin_persistence::prelude::*;

use crate::TradingEngineError;

pub struct DefaultEngine {
    pubsub: Arc<PubSub>,
    persistence: Arc<PersistenceService>,
    persistence_task: TaskTracker,
    persistence_shutdown: CancellationToken,

    service_tracker: TaskTracker,
    service_shutdown: CancellationToken,
}

impl DefaultEngine {
    pub async fn new() -> Self {
        // Init pubsub
        let pubsub = PubSub::new(1000000);

        // Init persistence
        let config = load::<PersistenceConfig>();
        let persistence = PersistenceService::new(pubsub.clone(), &config, false).await;
        let persistence_task = TaskTracker::new();
        let persistence_shutdown = CancellationToken::new();

        // Start persistence service
        let persistence_clone = persistence.clone();
        let persistence_shutdown_clone = persistence_shutdown.clone();
        persistence_task.spawn(async move {
            let res = persistence_clone.start(persistence_shutdown_clone).await;
            match res {
                Ok(_) => info!("Persistence service shutdown"),
                Err(e) => error!("Persistence service error: {:?}", e),
            }
        });

        Self {
            pubsub,
            persistence,
            persistence_task,
            persistence_shutdown,
            service_tracker: TaskTracker::new(),
            service_shutdown: CancellationToken::new(),
        }
    }

    pub async fn start_ingestor(&self, args: &IngestorsCommands) -> Result<(), TradingEngineError> {
        let config = load::<IngestorsConfig>();
        let ingestor = IngestorFactory::init(self.pubsub.clone(), self.persistence.clone(), &config.ingestors, args)?;

        let ingestor_clone = ingestor.clone();
        let ingestor_shutdown = self.service_shutdown.clone();

        let pubsub_clone = self.pubsub.clone();
        self.service_tracker.spawn(async move {
            let res = ingestor_clone.start(ingestor_shutdown).await;
            match res {
                Ok(_) => {
                    info!("Ingestor service finished");
                    pubsub_clone.publish(Event::Finished).await;
                }
                Err(e) => error!("Ingestor service error: {:?}", e),
            }
        });

        Ok(())
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

        self.persistence_shutdown.cancel();
        self.persistence_task.close();
        self.persistence_task.wait().await;
    }
}
