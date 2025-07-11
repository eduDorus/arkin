use std::time::Duration;

use arkin_audit::Audit;
use arkin_core::prelude::*;
use tracing::info;

#[tokio::test]
#[test_log::test]
async fn test_simulation() {
    info!("Starting simulation test...");
    let time = MockTime::new();

    let pubsub = PubSub::new(time.clone(), true);
    let pubsub_service = Service::new(pubsub.clone(), None);

    let audit = Audit::new("audit");
    let audit_service = Service::new(audit.to_owned(), Some(pubsub.subscribe(EventFilter::All)));

    let demo_publisher = pubsub.publisher();

    let engine = Engine::new();
    engine.register(pubsub_service, 0, 10).await;
    engine.register(audit_service, 0, 10).await;
    engine.start().await;

    // Publish some demo events
    for _ in 0..5 {
        demo_publisher
            .publish(Event::Finished(time.now().await + Duration::from_secs(1)))
            .await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        // time.advance_time_by(Duration::from_secs(1)).await;
    }

    info!(target: "integration-test", " --- LOG REVIEW ---");
    let event_log = audit.event_log();
    for event in event_log {
        info!(target: "integration-test", " - {} Event: {}",event.timestamp(), event.event_type());
    }

    engine.stop().await;
}
