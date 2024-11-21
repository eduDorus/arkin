use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use futures_util::stream;
use futures_util::StreamExt;
use mimalloc::MiMalloc;
use time::macros::datetime;
use tokio_rustls::rustls::crypto::aws_lc_rs;
use tokio_rustls::rustls::crypto::CryptoProvider;
use tracing::error;
use tracing::info;

use arkin_core::prelude::*;
use arkin_insights::prelude::*;
use arkin_persistence::prelude::*;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    init_tracing();
    info!("Starting Arkin Insights 🚀");

    // Install the default CryptoProvider
    CryptoProvider::install_default(aws_lc_rs::default_provider()).expect("Failed to install default CryptoProvider");

    let config = load::<PersistenceConfig>();
    let persistence_service = Arc::new(PersistenceService::from_config(&config));

    let config = load::<InsightsConfig>();
    let insights_service = Arc::new(InsightsService::from_config(
        &config.insights_service,
        persistence_service.clone(),
    ));

    // Fetch instruments concurrently
    let venue_symbols = vec!["BTCUSDT", "ETHUSDT", "SOLUSDT"];
    let fetches = venue_symbols.iter().map(|&s| {
        let service = Arc::clone(&persistence_service);
        async move {
            match service.read_instrument_by_venue_symbol(s.to_string()).await {
                Ok(instr) => Some(instr),
                Err(e) => {
                    error!("Failed to read instrument {}: {}", s, e);
                    None
                }
            }
        }
    });

    let results = stream::iter(fetches).buffer_unordered(5).collect::<Vec<_>>().await;
    let instruments: Vec<_> = results.into_iter().filter_map(|x| x).collect();

    if instruments.is_empty() {
        error!("No instruments loaded. Exiting.");
        return Ok(());
    }
    info!("Loaded {} instruments.", instruments.len());

    let start = datetime!(2024-10-01 00:00).assume_utc();
    let end = datetime!(2024-11-05 00:00).assume_utc();
    let frequency_secs = Duration::from_secs(60);

    let mut clock = Clock::new(start, end, frequency_secs);

    // Warm up pipeline state
    let trades = persistence_service.read_trades_range(&instruments, start, end).await?;
    // let ticks = persistence_service.read_ticks_range(&instruments, start, end).await?;
    info!("Loaded {} trades", trades.len());

    // Transform data to insights and add to state
    info!("Inserting trades and ticks into insights service");
    info!("Processing insights");
    let trade_insights = trades
        .into_iter()
        .map(|trade| trade.to_insights())
        .flatten()
        .collect::<Vec<_>>();
    // let tick_insights = ticks.into_iter().map(|tick| tick.to_insights()).flatten().collect::<Vec<_>>();
    info!("Done transforming data to insights");

    info!("Inserting insights into state");
    let _ = insights_service.insert_batch(trade_insights).await;
    // insights_service.insert_batch(tick_insights);
    info!("Done inserting insights into state");

    while let Some((_tick_start, tick_end)) = clock.next() {
        insights_service.process(&instruments, tick_end).await?;
    }

    persistence_service.flush().await?;
    Ok(())
}
