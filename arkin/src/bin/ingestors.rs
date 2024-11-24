use std::sync::Arc;

use mimalloc::MiMalloc;
use tokio_rustls::rustls::crypto::aws_lc_rs;
use tokio_rustls::rustls::crypto::CryptoProvider;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::error;
use tracing::info;

use arkin_core::prelude::*;
use arkin_ingestors::prelude::*;
use arkin_persistence::prelude::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();
    info!("Starting Arkin Ingestors 🚀");

    // Install the default CryptoProvider
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");

    let pubsub = Arc::new(PubSub::new());

    let config = load::<PersistenceConfig>();
    let persistence_service = Arc::new(PersistenceService::from_config(&config, pubsub.clone()));

    let config = load::<IngestorsConfig>();
    let ingestors = IngestorFactory::from_config(&config, pubsub.clone(), persistence_service.clone());

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

    // Start the persistence service
    let persistence_task_tracker = TaskTracker::new();
    let persistence_shutdown = CancellationToken::new();
    let shutdown = persistence_shutdown.clone();
    persistence_task_tracker.spawn(async move {
        if let Err(e) = persistence_service.start(shutdown).await {
            error!("Failed to start persistence service: {}", e);
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
    info!("All ingestors have shut down");

    persistence_shutdown.cancel();
    persistence_task_tracker.close();
    persistence_task_tracker.wait().await;
    info!("Persistence service has shut down");
}
