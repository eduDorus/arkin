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
async fn test_integration_bybit_spot_trades() {
    info!("Testing Bybit Spot Trades integration");

    let start = utc_datetime!(2024 - 01 - 01 00:00:00);
    let end = utc_datetime!(2024 - 01 - 01 00:01:00);

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
        info!("Bybit Spot Event: {}: {:?}", ts, parsed);
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
async fn test_integration_bybit_spot_book_ticker() {
    info!("Testing Bybit Spot Book Ticker integration");

    let start = utc_datetime!(2024 - 01 - 01 00:00:00);
    let end = utc_datetime!(2024 - 01 - 01 00:01:00);

    let venue = Exchange::BybitSpot;
    let channel = Channel::Ticker;
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

    // Collect and validate the events
    let mut event_count = 0;
    while let Some((_ts, json)) = stream.next().await {
        // Parse the stream event
        match parse_stream_event(&json, &venue, &channel) {
            Ok(stream_event) => {
                // Validate the parsed event
                assert_eq!(stream_event.venue, venue);
                assert_eq!(stream_event.channel, channel);
                assert_eq!(stream_event.instrument, "BTCUSDT");

                // Validate the tick data
                match stream_event.data {
                    ExchangeEventData::Tick(tick) => {
                        info!("Debug - tick values: bid_price={}, bid_quantity={}, ask_price={}, ask_quantity={}",
                              tick.bid_price, tick.bid_quantity, tick.ask_price, tick.ask_quantity);
                        assert!(tick.bid_price > rust_decimal::Decimal::ZERO);
                        assert!(tick.bid_quantity >= rust_decimal::Decimal::ZERO);
                        assert!(tick.ask_price > rust_decimal::Decimal::ZERO);
                        assert!(tick.ask_quantity >= rust_decimal::Decimal::ZERO);
                        assert!(tick.ask_price >= tick.bid_price); // Ask should be >= bid
                        info!(
                            "Parsed Bybit Spot Book Ticker: bid={}/{}, ask={}/{}",
                            tick.bid_price, tick.bid_quantity, tick.ask_price, tick.ask_quantity
                        );
                        event_count += 1;
                        if event_count >= 3 {
                            // Limit output for testing
                            break;
                        }
                    }
                    _ => panic!("Expected Tick data, got {:?}", stream_event.data),
                }
            }
            Err(e) => {
                // Log the raw JSON for debugging
                info!("Failed to parse JSON: {}", json);
                // Skip delta messages with incomplete data - this is expected
                if e.to_string().contains("Skipping Bybit orderbook delta message") ||
                   e.to_string().contains("Invalid orderbook data") {
                    continue;
                } else {
                    panic!("Failed to parse stream event: {}", e);
                }
            }
        }
    }
    assert!(event_count > 0, "Should have received at least one event");
}

#[tokio::test]
#[test_log::test]
async fn test_integration_bybit_derivatives_trades() {
    info!("Testing Bybit Derivatives Trades integration");

    let start = utc_datetime!(2024 - 01 - 01 00:00:00);
    let end = utc_datetime!(2024 - 01 - 01 00:01:00);

    let venue = Exchange::BybitDerivatives;
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
        info!("Bybit Derivatives Event: {}: {:?}", ts, parsed);
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
async fn test_integration_bybit_derivatives_book_ticker() {
    info!("Testing Bybit Derivatives Book Ticker integration");

    let start = utc_datetime!(2024 - 01 - 01 00:00:00);
    let end = utc_datetime!(2024 - 01 - 01 00:01:00);

    let venue = Exchange::BybitDerivatives;
    let channel = Channel::Ticker;
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

    // Collect and validate the events
    let mut event_count = 0;
    while let Some((_ts, json)) = stream.next().await {
        // Parse the stream event
        match parse_stream_event(&json, &venue, &channel) {
            Ok(stream_event) => {
                // Validate the parsed event
                assert_eq!(stream_event.venue, venue);
                assert_eq!(stream_event.channel, channel);
                assert_eq!(stream_event.instrument, "BTCUSDT");

                // Validate the tick data
                match stream_event.data {
                    ExchangeEventData::Tick(tick) => {
                        assert!(tick.bid_price > rust_decimal::Decimal::ZERO);
                        assert!(tick.bid_quantity >= rust_decimal::Decimal::ZERO);
                        assert!(tick.ask_price > rust_decimal::Decimal::ZERO);
                        assert!(tick.ask_quantity >= rust_decimal::Decimal::ZERO);
                        assert!(tick.ask_price >= tick.bid_price); // Ask should be >= bid
                        info!(
                            "Parsed Bybit Derivatives Book Ticker: bid={}/{}, ask={}/{}",
                            tick.bid_price, tick.bid_quantity, tick.ask_price, tick.ask_quantity
                        );
                        event_count += 1;
                        if event_count >= 3 {
                            // Limit output for testing
                            break;
                        }
                    }
                    _ => panic!("Expected Tick data, got {:?}", stream_event.data),
                }
            }
            Err(e) => {
                // Skip delta messages with incomplete data - this is expected
                if e.to_string().contains("Skipping Bybit orderbook delta message") ||
                   e.to_string().contains("Invalid orderbook data") {
                    continue;
                } else {
                    panic!("Failed to parse stream event: {}", e);
                }
            }
        }
    }
    assert!(event_count > 0, "Should have received at least one event");
}