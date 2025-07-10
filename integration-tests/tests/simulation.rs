use std::time::Duration;

use arkin_audit::Audit;
use arkin_core::prelude::*;
use tracing::info;

#[tokio::test]
#[test_log::test]
async fn test_simulation() {
    info!("Starting simulation test...");
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
}
