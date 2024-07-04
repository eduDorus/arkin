use anyhow::Result;
use aurelion::{
    data_providers::{binance::BinanceDataProvider, DataProvider},
    logging,
};
use mimalloc::MiMalloc;
use tracing::info;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    logging::init_tracing();
    info!("Starting Aurelion 🚀");

    let binance = BinanceDataProvider::new();

    let (tx, _rx) = flume::unbounded();

    binance.start(tx).await;

    Ok(())
}
