use tokio_rustls::rustls::crypto::{ring, CryptoProvider};
use tracing::info;

use arkin_core::prelude::*;
use arkin_engine::prelude::*;

#[tokio::main(flavor = "current_thread")]
// #[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();

    // Install the default CryptoProvider
    CryptoProvider::install_default(ring::default_provider()).expect("Failed to install default CryptoProvider");

    info!("Starting arkin 🚀");
    let engine = DefaultEngine::new().await;
    engine.wait_for_shutdown().await;
}
