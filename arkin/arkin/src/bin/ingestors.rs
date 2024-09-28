use std::sync::Arc;

use mimalloc::MiMalloc;
use tokio_rustls::rustls::crypto::aws_lc_rs;
use tokio_rustls::rustls::crypto::CryptoProvider;
use tracing::info;

use arkin_core::prelude::*;
use arkin_ingestors::prelude::*;
use arkin_persistance::prelude::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();
    info!("Starting Arkin 🚀");

    // Install the default CryptoProvider
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");

    let config = load::<PersistanceConfig>();
    let persistance_service = Arc::new(PersistanceService::from_config(&config.database));

    let config = load::<IngestorConfig>();
    let service = IngestorService::from_config(&config.ingestor_service, persistance_service);
    service.start().await;
}
