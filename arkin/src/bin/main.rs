use std::time::Duration;

use tokio_rustls::rustls::crypto::{ring, CryptoProvider};
use tracing::info;

use arkin_core::prelude::*;

#[tokio::main(flavor = "current_thread")]
// #[tokio::main(flavor = "multi_thread")]
async fn main() {
    init_tracing();

    // Install the default CryptoProvider
    CryptoProvider::install_default(ring::default_provider()).expect("Failed to install default CryptoProvider");

    info!("Starting arkin 🚀");

    let time = LiveSystemTime::new();
    let pubsub = PubSub::new(time, true);
    let pubsub_service = Service::new(pubsub.clone(), None);

    let engine = Engine::new();
    engine.register(pubsub_service, 0, 10).await;
    engine.start().await;
    tokio::time::sleep(Duration::from_secs(5)).await;
    engine.stop().await;
    // let engine = DefaultEngine::new().await;
    // engine.wait_for_shutdown().await;
}
