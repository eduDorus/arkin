use std::sync::Arc;

use mimalloc::MiMalloc;
use tokio_rustls::rustls::crypto::aws_lc_rs;
use tokio_rustls::rustls::crypto::CryptoProvider;
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

    let config = load::<PersistenceConfig>();
    let persistence_service = Arc::new(PersistenceService::from_config(&config));

    let config = load::<IngestorConfig>();
    let ingestor_service = IngestorService::from_config(&config.ingestor_service, persistence_service.clone());
    ingestor_service.start().await;

    if let Err(e) = persistence_service.flush().await {
        error!("Failed to flush persistence service: {}", e);
    }
}
