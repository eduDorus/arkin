use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use mimalloc::MiMalloc;
use time::macros::datetime;
use tokio_rustls::rustls::crypto::aws_lc_rs;
use tokio_rustls::rustls::crypto::CryptoProvider;
use tracing::info;

use arkin_core::prelude::*;
use arkin_insights::prelude::*;
use arkin_persistance::prelude::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    init_tracing();
    info!("Starting Arkin 🚀");

    // Install the default CryptoProvider
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");

    let config = load::<PersistanceConfig>();
    let persistance_service = Arc::new(PersistanceService::from_config(&config.database));

    let config = load::<InsightsConfig>();
    let insights_service = InsightsService::from_config(&config.insights_service, persistance_service.clone());

    let instruments = persistance_service
        .read_instrument_by_venue_symbol("BTCUSDT")
        .await?
        .expect("Instrument not found");

    let start = datetime!(2024-09-29 03:30).assume_utc();
    let end = datetime!(2024-09-29 12:15).assume_utc();
    let frequency_secs = Duration::from_secs(60);

    let mut clock = Clock::new(&start, &end, &frequency_secs);
    while let Some((from, to)) = clock.next() {
        insights_service.process(&[instruments.clone()], &from, &to).await?;
    }
    Ok(())
}
