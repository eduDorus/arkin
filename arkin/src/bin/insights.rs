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
    info!("Starting Arkin Insights 🚀");

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
    let instrument_ids = vec![instruments.id];

    let start = datetime!(2024-09-29 10:00).assume_utc();
    let end = datetime!(2024-09-29 18:00).assume_utc();
    let frequency_secs = Duration::from_secs(60);

    let mut clock = Clock::new(&start, &end, &frequency_secs);

    // Warm up pipeline state
    let trades = persistance_service.read_trades_range(&instrument_ids, &start, &end).await?;
    let ticks = persistance_service.read_ticks_range(&instrument_ids, &start, &end).await?;
    info!("Loaded {} trades and {} ticks", trades.len(), ticks.len());

    // Transform data to insights and add to state
    info!("Inserting trades and ticks into insights service");
    info!("Processing insights");
    let trade_insights = trades
        .into_iter()
        .map(|trade| trade.to_insights())
        .flatten()
        .collect::<Vec<_>>();
    let tick_insights = ticks.into_iter().map(|tick| tick.to_insights()).flatten().collect::<Vec<_>>();
    info!("Done transforming data to insights");

    info!("Inserting insights into state");
    insights_service.insert_batch(trade_insights);
    insights_service.insert_batch(tick_insights);
    info!("Done inserting insights into state");

    while let Some((from, to)) = clock.next() {
        insights_service.process(&[instruments.clone()], &from, &to).await?;
    }
    Ok(())
}
