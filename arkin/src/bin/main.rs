use std::time::Duration;

use arkin_audit::Audit;
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

    let pubsub = PubSub::new(time.clone(), true);
    let pubsub_service = Service::new(pubsub.clone(), None);

    let audit = Audit::new("audit");
    let audit_service = Service::new(audit, Some(pubsub.subscribe(EventFilter::All)));

    let demo_publisher = pubsub.publisher();

    let engine = Engine::new();
    engine.register(pubsub_service, 0, 10).await;
    engine.register(audit_service, 0, 10).await;
    engine.start().await;

    // Publish some demo events
    for _ in 0..5 {
        demo_publisher.publish(Event::Finished(time.now().await)).await;
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    engine.stop().await;
    // let engine = DefaultEngine::new().await;
    // engine.wait_for_shutdown().await;
}
