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
async fn test_integration_okx_spot_agg_trades() {
    info!("Testing OKX Spot AggTrades integration");

    let start = utc_datetime!(2025 - 01 - 01 00:00:00);
    let end = utc_datetime!(2025 - 01 - 01 00:01:00);

    let venue = VenueName::OkxSpot;
    let channel = Channel::AggTrades;
    let instruments = vec!["BTC-USDT".to_string()];

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
        info!("OKX Spot Event: {}: {:?}", ts, parsed);
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
async fn test_integration_okx_spot_book_ticker() {
    info!("Testing OKX Spot Book Ticker integration");

    let start = utc_datetime!(2024 - 01 - 01 00:00:00);
    let end = utc_datetime!(2024 - 01 - 01 00:01:00);

    let venue = VenueName::OkxSpot;
    let channel = Channel::Ticker;
    let instruments = vec!["BTC-USDT".to_string()];

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

    // Collect and validate the events
    let mut event_count = 0;
    while let Some((_ts, json)) = stream.next().await {
        // Parse the stream event
        let stream_event = parse_stream_event(&json, &venue, &channel).expect("Failed to parse stream event");

        // Validate the parsed event
        assert_eq!(stream_event.venue, venue);
        assert_eq!(stream_event.channel, channel);
        assert_eq!(stream_event.instrument, "BTC-USDT");

        // Validate the tick data
        match stream_event.data {
            ExchangeEventData::Tick(tick) => {
                assert!(tick.bid_price > rust_decimal::Decimal::ZERO);
                assert!(tick.bid_quantity >= rust_decimal::Decimal::ZERO);
                assert!(tick.ask_price > rust_decimal::Decimal::ZERO);
                assert!(tick.ask_quantity >= rust_decimal::Decimal::ZERO);
                assert!(tick.ask_price >= tick.bid_price); // Ask should be >= bid
                info!(
                    "Parsed OKX Spot Book Ticker: bid={}/{}, ask={}/{}",
                    tick.bid_price, tick.bid_quantity, tick.ask_price, tick.ask_quantity
                );
            }
            _ => panic!("Expected Tick data, got {:?}", stream_event.data),
        }

        event_count += 1;
        if event_count >= 3 {
            // Limit output for testing
            break;
        }
    }
    assert!(event_count > 0, "Should have received at least one event");
}

#[tokio::test]
#[test_log::test]
async fn test_integration_okx_swap_agg_trades() {
    info!("Testing OKX Swap AggTrades integration");

    let start = utc_datetime!(2025 - 01 - 01 00:00:00);
    let end = utc_datetime!(2025 - 01 - 01 00:01:00);

    let venue = VenueName::OkxSwap;
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
        info!("OKX Swap Event: {}: {:?}", ts, parsed);
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
async fn test_integration_okx_swap_book_ticker() {
    info!("Testing OKX Swap Book Ticker integration");

    let start = utc_datetime!(2024 - 01 - 01 00:00:00);
    let end = utc_datetime!(2024 - 01 - 01 00:01:00);

    let venue = VenueName::OkxSwap;
    let channel = Channel::Ticker;
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

    // Collect and validate the events
    let mut event_count = 0;
    while let Some((_ts, json)) = stream.next().await {
        // Parse the stream event
        let stream_event = parse_stream_event(&json, &venue, &channel).expect("Failed to parse stream event");

        // Validate the parsed event
        assert_eq!(stream_event.venue, venue);
        assert_eq!(stream_event.channel, channel);
        assert_eq!(stream_event.instrument, "BTC-USD-SWAP");

        // Validate the tick data
        match stream_event.data {
            ExchangeEventData::Tick(tick) => {
                assert!(tick.bid_price > rust_decimal::Decimal::ZERO);
                assert!(tick.bid_quantity >= rust_decimal::Decimal::ZERO);
                assert!(tick.ask_price > rust_decimal::Decimal::ZERO);
                assert!(tick.ask_quantity >= rust_decimal::Decimal::ZERO);
                assert!(tick.ask_price >= tick.bid_price); // Ask should be >= bid
                info!(
                    "Parsed OKX Swap Book Ticker: bid={}/{}, ask={}/{}",
                    tick.bid_price, tick.bid_quantity, tick.ask_price, tick.ask_quantity
                );
            }
            _ => panic!("Expected Tick data, got {:?}", stream_event.data),
        }

        event_count += 1;
        if event_count >= 3 {
            // Limit output for testing
            break;
        }
    }
    assert!(event_count > 0, "Should have received at least one event");
}

#[tokio::test]
#[test_log::test]
async fn test_integration_okx_swap_open_interest() {
    info!("Testing OKX Swap Open Interest integration");

    let start = utc_datetime!(2025 - 01 - 01 00:00:00);
    let end = utc_datetime!(2025 - 01 - 01 00:01:00);

    let venue = VenueName::OkxSwap;
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
        info!("OKX Swap Event: {}: {:?}", ts, parsed);
        event_count += 1;
        if event_count >= 5 {
            // Limit output for testing
            break;
        }
    }
    assert!(event_count > 0, "Should have received at least one event");
}
