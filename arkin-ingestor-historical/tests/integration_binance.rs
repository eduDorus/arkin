use std::sync::Arc;

use futures::StreamExt;
use time::macros::utc_datetime;
use tokio::pin;
use tracing::info;

use arkin_core::prelude::*;
use arkin_ingestor_historical::parse_stream_event;
use arkin_ingestor_historical::prelude::*;

#[tokio::test]
#[test_log::test]
async fn test_integration_binance_spot_agg_trades() {
    info!("Testing Binance integration");

    let start = utc_datetime!(2025 - 01 - 01 00:00:00);
    let end = utc_datetime!(2025 - 01 - 01 00:01:00);

    let venue = Exchange::BinanceSpot;
    let channel = Channel::AggTrades;
    let instruments = vec!["btcusdt".to_string()];

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
    let mut event_count = 0;
    while let Some((ts, json)) = stream.next().await {
        let parsed: ExchangeStreamEvent = parse_stream_event(&json, &venue, &channel).unwrap();
        info!("Binance Event: {}: {:?}", ts, parsed);
        event_count += 1;
        if event_count >= 5 {
            // Limit output for testing
            break;
        }
    }
    assert!(event_count > 0, "Should have received at least one event");
}

#[tokio::test]
#[test_log::test]
async fn test_integration_binance_usdm_agg_trades() {
    info!("Testing Binance integration");

    let start = utc_datetime!(2025 - 01 - 01 00:00:00);
    let end = utc_datetime!(2025 - 01 - 01 00:01:00);

    let venue = Exchange::BinanceUsdmFutures;
    let channel = Channel::AggTrades;
    let instruments = vec!["btcusdt".to_string()];

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
    let mut event_count = 0;
    while let Some((ts, json)) = stream.next().await {
        let parsed: ExchangeStreamEvent = parse_stream_event(&json, &venue, &channel).unwrap();
        info!("Binance Event: {}: {:?}", ts, parsed);
        event_count += 1;
        if event_count >= 5 {
            // Limit output for testing
            break;
        }
    }
    assert!(event_count > 0, "Should have received at least one event");
}

#[tokio::test]
#[test_log::test]
async fn test_integration_binance_usdm_open_interest() {
    info!("Testing Binance integration");

    let start = utc_datetime!(2025 - 01 - 01 00:00:00);
    let end = utc_datetime!(2025 - 01 - 01 00:01:00);

    let venue = Exchange::BinanceUsdmFutures;
    let channel = Channel::OpenInterest;
    let instruments = vec!["btcusdt".to_string()];

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
    let mut event_count = 0;
    while let Some((ts, json)) = stream.next().await {
        info!("Raw JSON: {}", json);
        let parsed: ExchangeStreamEvent = parse_stream_event(&json, &venue, &channel).unwrap();
        info!("Binance Event: {}: {:?}", ts, parsed);
        event_count += 1;
        if event_count >= 5 {
            // Limit output for testing
            break;
        }
    }
    assert!(event_count > 0, "Should have received at least one event");
}

#[tokio::test]
#[test_log::test]
async fn test_integration_bybit() {
    info!("Testing Bybit integration");

    let start = utc_datetime!(2025 - 01 - 01 00:00:00);
    let end = utc_datetime!(2025 - 01 - 01 00:01:00);

    let venue = Exchange::BybitSpot;
    let channel = Channel::Trades;
    let instruments = vec!["BTCUSDT".to_string()];

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
    let mut event_count = 0;
    while let Some((ts, json)) = stream.next().await {
        let parsed: ExchangeStreamEvent = parse_stream_event(&json, &venue, &channel).unwrap();
        info!("Bybit Event: {}: {:?}", ts, parsed);
        event_count += 1;
        if event_count >= 5 {
            // Limit output for testing
            break;
        }
    }
    assert!(event_count > 0, "Should have received at least one event");
}

#[tokio::test]
#[test_log::test]
async fn test_integration_okx_spot_agg_trades() {
    info!("Testing OKX integration");

    let start = utc_datetime!(2025 - 01 - 01 00:00:00);
    let end = utc_datetime!(2025 - 01 - 01 00:01:00);

    let venue = Exchange::OkxSpot;
    let channel = Channel::AggTrades;
    let instruments = vec!["ETH-BTC".to_string()];

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
    let mut event_count = 0;
    while let Some((ts, json)) = stream.next().await {
        let parsed: ExchangeStreamEvent = parse_stream_event(&json, &venue, &channel).unwrap();
        info!("OKX Event: {}: {:?}", ts, parsed);
        event_count += 1;
        if event_count >= 5 {
            // Limit output for testing
            break;
        }
    }
    assert!(event_count > 0, "Should have received at least one event");
}

#[tokio::test]
#[test_log::test]
async fn test_integration_okx_swap_agg_trades() {
    info!("Testing OKX integration");

    let start = utc_datetime!(2025 - 01 - 01 00:00:00);
    let end = utc_datetime!(2025 - 01 - 01 00:01:00);

    let venue = Exchange::OkxSwap;
    let channel = Channel::AggTrades;
    let instruments = vec!["BTC-USD-SWAP".to_string()];

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
    let mut event_count = 0;
    while let Some((ts, json)) = stream.next().await {
        let parsed: ExchangeStreamEvent = parse_stream_event(&json, &venue, &channel).unwrap();
        info!("OKX Event: {}: {:?}", ts, parsed);
        event_count += 1;
        if event_count >= 5 {
            // Limit output for testing
            break;
        }
    }
    assert!(event_count > 0, "Should have received at least one event");
}

#[tokio::test]
#[test_log::test]
async fn test_integration_okx_swap_open_interest() {
    info!("Testing OKX integration");

    let start = utc_datetime!(2025 - 01 - 01 00:00:00);
    let end = utc_datetime!(2025 - 01 - 01 00:01:00);

    let venue = Exchange::OkxSwap;
    let channel = Channel::OpenInterest;
    let instruments = vec!["BTC-USD-SWAP".to_string()];

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
    let mut event_count = 0;
    while let Some((ts, json)) = stream.next().await {
        info!("Raw JSON: {}", json);
        let parsed: ExchangeStreamEvent = parse_stream_event(&json, &venue, &channel).unwrap();
        info!("OKX Event: {}: {:?}", ts, parsed);
        event_count += 1;
        if event_count >= 5 {
            // Limit output for testing
            break;
        }
    }
    assert!(event_count > 0, "Should have received at least one event");
}
