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

#[tokio::test]
#[test_log::test]
async fn test_integration_binance_usdm_long_short_ratio() {
    info!("Testing Binance USDM Long/Short Ratio integration");

    let start = utc_datetime!(2024 - 01 - 01 00:00:00);
    let end = utc_datetime!(2024 - 01 - 01 00:01:00);

    let venue = Exchange::BinanceUsdmFutures;
    let channel = Channel::LongShortRatio;
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
        // For now, just log the raw data since parsing might not be implemented yet
        info!("Binance USDM Long/Short Ratio Event: {}: {}", ts, json);
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
async fn test_integration_binance_coinm_long_short_ratio() {
    info!("Testing Binance COIN Long/Short Ratio integration");

    let start = utc_datetime!(2024 - 01 - 01 00:00:00);
    let end = utc_datetime!(2024 - 01 - 01 00:01:00);

    let venue = Exchange::BinanceCoinmFutures;
    let channel = Channel::LongShortRatio;
    let instruments = vec!["btcusd".to_string()];

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
        // For now, just log the raw data since parsing might not be implemented yet
        info!("Binance COIN Long/Short Ratio Event: {}: {}", ts, json);
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
async fn test_integration_bybit_long_short_ratio() {
    info!("Testing Bybit Long/Short Ratio integration");

    let start = utc_datetime!(2025 - 01 - 01 00:00:00);
    let end = utc_datetime!(2025 - 01 - 01 00:01:00);

    let venue = Exchange::BybitDerivatives;
    let channel = Channel::LongShortRatio;
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
        info!("Raw JSON: {}", json);
        // For now, just log the raw data since parsing might not be implemented yet
        info!("Bybit Long/Short Ratio Event: {}: {}", ts, json);
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
async fn test_integration_binance_spot_book_ticker() {
    info!("Testing Binance Spot Book Ticker integration");

    let start = utc_datetime!(2024 - 01 - 01 00:00:00);
    let end = utc_datetime!(2024 - 01 - 01 00:01:00);

    let venue = Exchange::BinanceSpot;
    let channel = Channel::Ticker;
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
        // For now, just log the raw data since parsing might not be implemented yet
        info!("Binance Spot Book Ticker Event: {}: {}", ts, json);
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
async fn test_integration_binance_usdm_book_ticker() {
    info!("Testing Binance USDM Futures Book Ticker integration");

    let start = utc_datetime!(2024 - 01 - 01 00:00:00);
    let end = utc_datetime!(2024 - 01 - 01 00:01:00);

    let venue = Exchange::BinanceUsdmFutures;
    let channel = Channel::Ticker;
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

    // Collect and validate the events
    let mut event_count = 0;
    while let Some((_ts, json)) = stream.next().await {
        // Parse the stream event
        let stream_event = parse_stream_event(&json, &venue, &channel)
            .expect("Failed to parse stream event");

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
                info!("Parsed Binance USDM Book Ticker: bid={}/{}, ask={}/{}",
                      tick.bid_price, tick.bid_quantity, tick.ask_price, tick.ask_quantity);
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
async fn test_integration_binance_coinm_book_ticker() {
    info!("Testing Binance COIN Futures Book Ticker integration");

    let start = utc_datetime!(2024 - 01 - 01 00:00:00);
    let end = utc_datetime!(2024 - 01 - 01 00:01:00);

    let venue = Exchange::BinanceCoinmFutures;
    let channel = Channel::Ticker;
    let instruments = vec!["btcusd_perp".to_string()];

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
        let stream_event = parse_stream_event(&json, &venue, &channel)
            .expect("Failed to parse stream event");

        // Validate the parsed event
        assert_eq!(stream_event.venue, venue);
        assert_eq!(stream_event.channel, channel);
        assert_eq!(stream_event.instrument, "BTCUSD_PERP");

        // Validate the tick data
        match stream_event.data {
            ExchangeEventData::Tick(tick) => {
                assert!(tick.bid_price > rust_decimal::Decimal::ZERO);
                assert!(tick.bid_quantity >= rust_decimal::Decimal::ZERO);
                assert!(tick.ask_price > rust_decimal::Decimal::ZERO);
                assert!(tick.ask_quantity >= rust_decimal::Decimal::ZERO);
                assert!(tick.ask_price >= tick.bid_price); // Ask should be >= bid
                info!("Parsed Binance COIN Book Ticker: bid={}/{}, ask={}/{}",
                      tick.bid_price, tick.bid_quantity, tick.ask_price, tick.ask_quantity);
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
#[ignore] // Bybit spot may not provide book ticker streaming data
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
        let stream_event = parse_stream_event(&json, &venue, &channel)
            .expect("Failed to parse stream event");

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
                info!("Parsed Bybit Spot Book Ticker: bid={}/{}, ask={}/{}",
                      tick.bid_price, tick.bid_quantity, tick.ask_price, tick.ask_quantity);
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
        let stream_event = parse_stream_event(&json, &venue, &channel)
            .expect("Failed to parse stream event");

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
                info!("Parsed Bybit Derivatives Book Ticker: bid={}/{}, ask={}/{}",
                      tick.bid_price, tick.bid_quantity, tick.ask_price, tick.ask_quantity);
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
async fn test_integration_okx_spot_book_ticker() {
    info!("Testing OKX Spot Book Ticker integration");

    let start = utc_datetime!(2024 - 01 - 01 00:00:00);
    let end = utc_datetime!(2024 - 01 - 01 00:01:00);

    let venue = Exchange::OkxSpot;
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
        let stream_event = parse_stream_event(&json, &venue, &channel)
            .expect("Failed to parse stream event");

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
                info!("Parsed OKX Spot Book Ticker: bid={}/{}, ask={}/{}",
                      tick.bid_price, tick.bid_quantity, tick.ask_price, tick.ask_quantity);
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
async fn test_integration_okx_swap_book_ticker() {
    info!("Testing OKX Swap Book Ticker integration");

    let start = utc_datetime!(2024 - 01 - 01 00:00:00);
    let end = utc_datetime!(2024 - 01 - 01 00:01:00);

    let venue = Exchange::OkxSwap;
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
        let stream_event = parse_stream_event(&json, &venue, &channel)
            .expect("Failed to parse stream event");

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
                info!("Parsed OKX Swap Book Ticker: bid={}/{}, ask={}/{}",
                      tick.bid_price, tick.bid_quantity, tick.ask_price, tick.ask_quantity);
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
