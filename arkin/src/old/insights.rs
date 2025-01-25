use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use time::macros::datetime;
use tokio_rustls::rustls::crypto::aws_lc_rs;
use tokio_rustls::rustls::crypto::CryptoProvider;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::error;
use tracing::info;

use arkin_core::prelude::*;
use arkin_insights::prelude::*;
use arkin_persistence::prelude::*;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    init_tracing();
    info!("Starting Arkin Insights 🚀");

    // Install the default CryptoProvider
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");

    let pubsub = Arc::new(PubSub::new());

    let config = load::<PersistenceConfig>();
    let persistence = Arc::new(PersistenceService::from_config(&config, pubsub.clone()).await);

    // Start the persistence service
    let persistence_task_tracker = TaskTracker::new();
    let persistence_shutdown = CancellationToken::new();
    let shutdown = persistence_shutdown.clone();
    let persistence_service = persistence.clone();
    persistence_task_tracker.spawn(async move {
        if let Err(e) = persistence_service.start(shutdown).await {
            error!("Failed to start persistence service: {}", e);
        }
    });

    let config = load::<InsightsConfig>().insights_service;
    let insights_service = Arc::new(InsightsService::from_config(&config, pubsub.clone(), persistence.clone()).await);

    // Fetch instruments concurrently
    let venue_symbols = vec!["BTCUSDT", "ETHUSDT", "SOLUSDT"];
    let mut instruments = vec![];
    for symbol in venue_symbols {
        match persistence.instrument_store.read_by_venue_symbol(symbol).await {
            Ok(instr) => instruments.push(instr),
            Err(e) => error!("Failed to read instrument {}: {}", symbol, e),
        }
    }

    info!("Loaded {} instruments.", instruments.len());

    let start = datetime!(2024-12-01 00:00).assume_utc();
    let end = datetime!(2024-12-24 00:00).assume_utc();

    let mut current_day = start.date();
    let frequency_secs = Duration::from_secs(config.frequency_secs);

    let mut clock = Clock::new(start, end, frequency_secs);

    while let Some((_tick_start, tick_end)) = clock.next() {
        if tick_end.date() != current_day {
            current_day = tick_end.date();

            // Remove the data
            insights_service.remove(tick_end).await?;

            // Load the data
            let tomorrow = tick_end + Duration::from_secs(86400);
            insights_service
                .load(tomorrow, &instruments, Duration::from_secs(86400))
                .await?;
        }
        insights_service.process(tick_end, &instruments, true).await?;
    }

    persistence.flush().await?;

    persistence_shutdown.cancel();
    persistence_task_tracker.close();
    persistence_task_tracker.wait().await;
    info!("Persistence service has shut down");
    Ok(())
}
