use std::sync::Arc;

use futures::StreamExt;
use time::macros::utc_datetime;
use tokio::pin;
use tracing::info;

use arkin_core::prelude::*;
use arkin_ingestor_historical::prelude::*;

#[tokio::test]
#[test_log::test]
async fn test_binance_spot() {
    info!("Hello world");

    let start = utc_datetime!(2025 - 01 - 01 00:00:00);
    let end = utc_datetime!(2025 - 01 - 01 00:01:00);

    // let venue = Exchange::BinanceSpot;
    // let venue = Exchange::BinanceUsdmFutures;
    // let channel = Channel::AggTrades;
    // let instruments = vec!["btcusdt".to_string()];

    // let venue = Exchange::BybitSpot;
    // let venue = Exchange::BybitDerivatives;
    // let channel = Channel::Trades;
    // let instruments = vec!["BTCUSDT".to_string()];

    // let venue = Exchange::OkxSpot;
    let venue = Exchange::OkxSwap;
    let channel = Channel::AggTrades;
    // let instruments = vec!["ETH-BTC".to_string()];
    let instruments = vec!["BTC-USDT-SWAP".to_string()];

    let cfg = load::<TardisConfig>();
    let ingestor = Arc::new(
        TardisIngestor::builder()
            .venue(venue.clone())
            .channel(channel.clone())
            .start(start)
            .end(end)
            .instruments(instruments.clone())
            .max_concurrent_requests(1)
            .base_url(cfg.tardis.http_url)
            .api_secret(None)
            .build(),
    );

    let req = TardisRequest::new(
        ingestor.venue.clone(),
        ingestor.channel.clone(),
        ingestor.instruments.clone(),
        ingestor.start,
        ingestor.end,
    );

    let stream = ingestor.stream(req);
    pin!(stream);

    // Collect and print the events for demonstration purposes
    while let Some((ts, json)) = stream.next().await {
        let parsed: RawMarketData = serde_json::from_str(&json).unwrap();
        info!("Event: {}: {:?}", ts, parsed);
    }
}
