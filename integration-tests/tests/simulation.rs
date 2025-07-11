use std::{str::FromStr, sync::Arc, time::Duration};

use arkin_accounting::Accounting;
use arkin_audit::Audit;
use arkin_core::prelude::*;
use arkin_ingestor_binance::SimBinanceIngestor;
use arkin_persistence::{Persistence, PersistenceConfig};
use tracing::info;
use uuid::Uuid;

#[tokio::test]
#[test_log::test]
async fn test_simulation() {
    info!("Starting simulation test...");
    // Init mock time
    let time = MockTime::new();

    // Start and end time
    let start_time = time.now().await;
    let end_time = start_time + Duration::from_secs(86400);

    // Init pubsub
    let pubsub = PubSub::new(time.clone(), true);
    let pubsub_service = Service::new(pubsub.clone(), None);

    // Init persistence
    let config = load::<PersistenceConfig>();
    let instance = Instance::builder()
        .id(Uuid::from_str("04432ac5-483d-46a3-811b-6be79d6ef7c1").unwrap())
        .name("integration-test".to_owned())
        .instance_type(InstanceType::Test)
        .build();
    let persistence = Persistence::new(&config, instance, false, false, true).await;
    let persistence_service = Service::new(persistence.to_owned(), Some(pubsub.subscribe(EventFilter::All)));

    // Init accounting
    let accounting = Arc::new(
        Accounting::builder()
            .time(time.to_owned())
            ._publisher(pubsub.publisher())
            .build(),
    );
    let accounting_service = Service::new(
        accounting.to_owned(),
        Some(pubsub.subscribe(EventFilter::Events(vec![EventType::VenueOrderFill]))),
    );

    // Init audit
    let audit = Audit::new("audit");
    let audit_service = Service::new(audit.to_owned(), Some(pubsub.subscribe(EventFilter::All)));

    // Init sim ingestor
    let binance_ingestor = Arc::new(
        SimBinanceIngestor::builder()
            .identifier("sim-binance-ingestor".to_owned())
            ._time(time.to_owned())
            .start(start_time)
            .end(end_time + Duration::from_secs(1))
            .instruments(vec![test_inst_binance_btc_usdt_perp()])
            .persistence(persistence.to_owned())
            .publisher(pubsub.publisher())
            .build(),
    );
    let binance_ingestor_service = Service::new(binance_ingestor, None);

    // Setup engine
    // let demo_publisher = pubsub.publisher();
    let engine = Engine::new();
    engine.register(pubsub_service, 0, 10).await;
    engine.register(persistence_service, 0, 10).await;
    engine.register(audit_service, 0, 10).await;
    engine.register(accounting_service, 0, 10).await;
    engine.register(binance_ingestor_service, 1, 9).await;

    engine.start().await;

    // Publish some demo events
    // for _ in 0..5 {
    //     demo_publisher
    //         .publish(Event::Finished(time.now().await + Duration::from_secs(1)))
    //         .await;
    //     tokio::time::sleep(Duration::from_secs(1)).await;
    //     // time.advance_time_by(Duration::from_secs(1)).await;
    // }

    while time.now().await < end_time {
        info!(target: "integration-test", "Current time: {} end time: {} diff: {}", time.now().await, end_time, end_time - time.now().await);
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    info!(target: "integration-test", " --- LOG REVIEW ---");
    let event_log = audit.event_log();
    info!(target: "integration-test", "received {} events", event_log.len());
    // for event in event_log {
    //     info!(target: "integration-test", " - {} Event: {}",event.timestamp(), event.event_type());
    // }

    engine.stop().await;
}
