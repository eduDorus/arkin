use std::str::FromStr;

use anyhow::Result;
use aurelion::{
    logging,
    providers::ws::{Subscription, WebSocketManager},
};
use mimalloc::MiMalloc;
use tracing::info;
use url::Url;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    logging::init_tracing();
    info!("Starting Aurelion 🚀");

    let url = Url::from_str("wss://fstream.binance.com/ws")?;

    let subscription = Subscription::new(vec!["btcusdt@aggTrade", "btcusdt@ticker"]);
    let mut ws = WebSocketManager::new(url, 5, 100, subscription).await?;

    let (rx, _tx) = flume::unbounded();
    ws.run(rx).await?;

    Ok(())
}
