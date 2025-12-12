use anyhow::Result;
use arkin_core::{Engine, InstanceType, LiveSystemTime, Subscriber};
use arkin_data_provider::DataProviderService;
use integration_tests::{init_test_persistence, init_test_pubsub};
use std::sync::Arc;
use tracing::info;

#[tokio::test]
#[test_log::test]
async fn test_data_provider_service() -> Result<()> {
    let clock = LiveSystemTime::new();
    let persistence = init_test_persistence().await;
    persistence.refresh().await?;
    let pubsub = init_test_pubsub().await;
    let receiver = pubsub.subscribe(arkin_core::EventFilter::All);
    tokio::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            info!("{}", msg);
            if receiver.needs_ack() {
                receiver.send_ack().await;
            }
        }
    });

    info!("Starting integration test: test_data_provider_service");

    let mut engine = Engine::new(clock, pubsub.clone(), persistence.clone(), InstanceType::Test);
    engine.register("pubsub", pubsub.clone(), 0, 10);
    engine.register("persistence", persistence.clone(), 0, 10);

    let data_provider_service = DataProviderService::load_config();
    engine.register("data_provider", Arc::new(data_provider_service), 1, 1);

    engine.start().await;

    // Sleep for a short duration to allow connections to establish
    // tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    engine.wait_for_shutdown().await;
    engine.stop().await;

    Ok(())
}
