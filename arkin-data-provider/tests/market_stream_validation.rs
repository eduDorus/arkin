use futures::{SinkExt, StreamExt};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info};

/// Market configuration (replicated from market_config.rs for integration test)
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Exchange {
    Binance,
    Okx,
    Bybit,
    Coinbase,
}

impl std::fmt::Display for Exchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Exchange::Binance => write!(f, "binance"),
            Exchange::Okx => write!(f, "okx"),
            Exchange::Bybit => write!(f, "bybit"),
            Exchange::Coinbase => write!(f, "coinbase"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum MarketType {
    Spot,
    Perpetual,        // USDT-margined perpetuals (Binance USDT-M, OKX USDT-SWAP, Bybit USDT, Coinbase)
    InversePerpetual, // Coin-margined perpetuals (Binance USDS-M, OKX USDC-SWAP, Bybit USDC)
}

impl std::fmt::Display for MarketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketType::Spot => write!(f, "spot"),
            MarketType::Perpetual => write!(f, "perpetual"),
            MarketType::InversePerpetual => write!(f, "inverse_perpetual"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StreamType {
    AggregateTrades,
    Trades,
    Ticker24h,
    TickerRealtime,
    MarkPrice,
    IndexPrice,
    Liquidations,
    FundingRate,
    OpenInterest,
    BestBidAsk,
}

impl std::fmt::Display for StreamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamType::AggregateTrades => write!(f, "aggregate_trades"),
            StreamType::Trades => write!(f, "trades"),
            StreamType::Ticker24h => write!(f, "ticker_24h"),
            StreamType::TickerRealtime => write!(f, "ticker_realtime"),
            StreamType::MarkPrice => write!(f, "mark_price"),
            StreamType::IndexPrice => write!(f, "index_price"),
            StreamType::Liquidations => write!(f, "liquidations"),
            StreamType::FundingRate => write!(f, "funding_rate"),
            StreamType::OpenInterest => write!(f, "open_interest"),
            StreamType::BestBidAsk => write!(f, "best_bid_ask"),
        }
    }
}

/// Test stream endpoint
#[derive(Clone)]
pub struct TestStream {
    pub exchange: Exchange,
    pub market_type: MarketType,
    pub stream_type: StreamType,
    pub url: String,
    pub subscription: String,
    pub symbol_placeholder: &'static str,
    pub test_symbol: String,
}

impl TestStream {
    fn subscription_with_symbol(&self) -> String {
        self.subscription
            .replace(&format!("<{}>", self.symbol_placeholder), &self.test_symbol)
    }

    fn display_name(&self) -> String {
        format!("{} {} {}", self.exchange, self.market_type, self.stream_type)
    }
}

/// Result of testing a single stream
#[derive(Clone, Debug)]
pub struct StreamTestResult {
    pub stream_name: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub connection_time_ms: u64,
    pub time_to_first_message_ms: Option<u64>,
    pub first_message_sample: Option<String>,
}

/// Test configuration
pub struct StreamValidationConfig {
    pub connection_timeout_secs: u64,
    pub message_timeout_secs: u64,
    pub test_streams: Vec<TestStream>,
}

impl StreamValidationConfig {
    pub fn new() -> Self {
        let mut test_streams = vec![];

        // ======== BINANCE SPOT ========
        test_streams.push(TestStream {
            exchange: Exchange::Binance,
            market_type: MarketType::Spot,
            stream_type: StreamType::AggregateTrades,
            url: "wss://stream.binance.com:9443/ws".to_string(),
            subscription: r#"{"method":"SUBSCRIBE","params":["<symbol>@aggTrade"],"id":1}"#.to_string(),
            symbol_placeholder: "symbol",
            test_symbol: "btcusdt".to_string(),
        });

        test_streams.push(TestStream {
            exchange: Exchange::Binance,
            market_type: MarketType::Spot,
            stream_type: StreamType::TickerRealtime,
            url: "wss://stream.binance.com:9443/ws".to_string(),
            subscription: r#"{"method":"SUBSCRIBE","params":["<symbol>@ticker"],"id":1}"#.to_string(),
            symbol_placeholder: "symbol",
            test_symbol: "btcusdt".to_string(),
        });

        // ======== BINANCE USDT PERPETUALS ========
        test_streams.push(TestStream {
            exchange: Exchange::Binance,
            market_type: MarketType::Perpetual,
            stream_type: StreamType::AggregateTrades,
            url: "wss://fstream.binance.com/ws".to_string(),
            subscription: r#"{"method":"SUBSCRIBE","params":["!<symbol>@aggTrade"],"id":1}"#.to_string(),
            symbol_placeholder: "symbol",
            test_symbol: "btcusdt".to_string(),
        });

        test_streams.push(TestStream {
            exchange: Exchange::Binance,
            market_type: MarketType::Perpetual,
            stream_type: StreamType::MarkPrice,
            url: "wss://fstream.binance.com/ws".to_string(),
            subscription: r#"{"method":"SUBSCRIBE","params":["!<symbol>@markPrice"],"id":1}"#.to_string(),
            symbol_placeholder: "symbol",
            test_symbol: "btcusdt".to_string(),
        });

        // ======== BINANCE USDS PERPETUALS ========
        test_streams.push(TestStream {
            exchange: Exchange::Binance,
            market_type: MarketType::InversePerpetual,
            stream_type: StreamType::AggregateTrades,
            url: "wss://fstream.binance.com/ws".to_string(),
            subscription: r#"{"method":"SUBSCRIBE","params":["!<symbol>@aggTrade"],"id":1}"#.to_string(),
            symbol_placeholder: "symbol",
            test_symbol: "btcusdt".to_string(),
        });

        test_streams.push(TestStream {
            exchange: Exchange::Binance,
            market_type: MarketType::InversePerpetual,
            stream_type: StreamType::MarkPrice,
            url: "wss://fstream.binance.com/ws".to_string(),
            subscription: r#"{"method":"SUBSCRIBE","params":["!<symbol>@markPrice"],"id":1}"#.to_string(),
            symbol_placeholder: "symbol",
            test_symbol: "btcusdt".to_string(),
        });

        // ======== OKX SPOT ========
        test_streams.push(TestStream {
            exchange: Exchange::Okx,
            market_type: MarketType::Spot,
            stream_type: StreamType::Trades,
            url: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
            subscription: r#"{"op":"subscribe","args":[{"channel":"trades","instId":"<instId>"}]}"#.to_string(),
            symbol_placeholder: "instId",
            test_symbol: "BTC-USDT".to_string(),
        });

        test_streams.push(TestStream {
            exchange: Exchange::Okx,
            market_type: MarketType::Spot,
            stream_type: StreamType::TickerRealtime,
            url: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
            subscription: r#"{"op":"subscribe","args":[{"channel":"tickers","instId":"<instId>"}]}"#.to_string(),
            symbol_placeholder: "instId",
            test_symbol: "BTC-USDT".to_string(),
        });

        // ======== OKX PERPETUALS ========
        test_streams.push(TestStream {
            exchange: Exchange::Okx,
            market_type: MarketType::Perpetual,
            stream_type: StreamType::Trades,
            url: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
            subscription: r#"{"op":"subscribe","args":[{"channel":"trades","instId":"<instId>"}]}"#.to_string(),
            symbol_placeholder: "instId",
            test_symbol: "BTC-USDT-SWAP".to_string(),
        });

        test_streams.push(TestStream {
            exchange: Exchange::Okx,
            market_type: MarketType::Perpetual,
            stream_type: StreamType::FundingRate,
            url: "wss://ws.okx.com:8443/ws/v5/public".to_string(),
            subscription: r#"{"op":"subscribe","args":[{"channel":"funding-rate","instId":"<instId>"}]}"#.to_string(),
            symbol_placeholder: "instId",
            test_symbol: "BTC-USDT-SWAP".to_string(),
        });

        // ======== BYBIT SPOT ========
        test_streams.push(TestStream {
            exchange: Exchange::Bybit,
            market_type: MarketType::Spot,
            stream_type: StreamType::Trades,
            url: "wss://stream.bybit.com/v5/public/spot".to_string(),
            subscription: r#"{"op":"subscribe","args":["publicTrade.<symbol>"]}"#.to_string(),
            symbol_placeholder: "symbol",
            test_symbol: "BTCUSDT".to_string(),
        });

        test_streams.push(TestStream {
            exchange: Exchange::Bybit,
            market_type: MarketType::Spot,
            stream_type: StreamType::TickerRealtime,
            url: "wss://stream.bybit.com/v5/public/spot".to_string(),
            subscription: r#"{"op":"subscribe","args":["tickers.<symbol>"]}"#.to_string(),
            symbol_placeholder: "symbol",
            test_symbol: "BTCUSDT".to_string(),
        });

        // ======== BYBIT PERPETUALS ========
        test_streams.push(TestStream {
            exchange: Exchange::Bybit,
            market_type: MarketType::Perpetual,
            stream_type: StreamType::Trades,
            url: "wss://stream.bybit.com/v5/public/linear".to_string(),
            subscription: r#"{"op":"subscribe","args":["publicTrade.<symbol>"]}"#.to_string(),
            symbol_placeholder: "symbol",
            test_symbol: "BTCUSDT".to_string(),
        });

        test_streams.push(TestStream {
            exchange: Exchange::Bybit,
            market_type: MarketType::Perpetual,
            stream_type: StreamType::MarkPrice,
            url: "wss://stream.bybit.com/v5/public/linear".to_string(),
            subscription: r#"{"op":"subscribe","args":["markPrice.<symbol>"]}"#.to_string(),
            symbol_placeholder: "symbol",
            test_symbol: "BTCUSDT".to_string(),
        });

        // ======== COINBASE SPOT ========
        test_streams.push(TestStream {
            exchange: Exchange::Coinbase,
            market_type: MarketType::Spot,
            stream_type: StreamType::Trades,
            url: "wss://advanced-trade-ws.coinbase.com".to_string(),
            subscription: r#"{"type":"subscribe","product_ids":["<product_id>"],"channel":"market_trades"}"#
                .to_string(),
            symbol_placeholder: "product_id",
            test_symbol: "BTC-USD".to_string(),
        });

        test_streams.push(TestStream {
            exchange: Exchange::Coinbase,
            market_type: MarketType::Spot,
            stream_type: StreamType::TickerRealtime,
            url: "wss://advanced-trade-ws.coinbase.com".to_string(),
            subscription: r#"{"type":"subscribe","product_ids":["<product_id>"],"channel":"ticker"}"#.to_string(),
            symbol_placeholder: "product_id",
            test_symbol: "BTC-USD".to_string(),
        });

        Self {
            connection_timeout_secs: 5,
            message_timeout_secs: 10,
            test_streams,
        }
    }
}

/// Test a single stream endpoint
pub async fn test_stream(stream: &TestStream, message_timeout: Duration) -> StreamTestResult {
    let stream_name = stream.display_name();
    let connection_start = Instant::now();

    // Connect to WebSocket
    match timeout(Duration::from_secs(5), connect_async(&stream.url)).await {
        Err(_) => {
            error!("  ✗ {} | {}", stream.exchange, stream.stream_type);
            error!("    → Connection timeout (5s)");
            return StreamTestResult {
                stream_name,
                success: false,
                error_message: Some("Connection timeout".to_string()),
                connection_time_ms: connection_start.elapsed().as_millis() as u64,
                time_to_first_message_ms: None,
                first_message_sample: None,
            };
        }
        Ok(Err(e)) => {
            error!("  ✗ {} | {} | {}", stream.exchange, stream.market_type, stream.stream_type);
            error!("    → Connection failed: {}", e);
            return StreamTestResult {
                stream_name,
                success: false,
                error_message: Some(format!("Connection failed: {}", e)),
                connection_time_ms: connection_start.elapsed().as_millis() as u64,
                time_to_first_message_ms: None,
                first_message_sample: None,
            };
        }
        Ok(Ok((ws_stream, _))) => {
            let connection_time = connection_start.elapsed();
            let (mut sink, mut msg_stream) = ws_stream.split();

            // Send subscription
            let subscription = stream.subscription_with_symbol();
            if let Err(e) = sink.send(Message::text(subscription)).await {
                error!("  ✗ {} | {} | {}", stream.exchange, stream.market_type, stream.stream_type);
                error!("    → Failed to send subscription: {}", e);
                return StreamTestResult {
                    stream_name,
                    success: false,
                    error_message: Some(format!("Failed to send subscription: {}", e)),
                    connection_time_ms: connection_time.as_millis() as u64,
                    time_to_first_message_ms: None,
                    first_message_sample: None,
                };
            }

            // Wait for first message
            let message_start = Instant::now();
            match timeout(message_timeout, msg_stream.next()).await {
                Err(_) => {
                    error!("  ✗ {} | {} | {}", stream.exchange, stream.market_type, stream.stream_type);
                    error!("    → No message received within {}s", message_timeout.as_secs());
                    StreamTestResult {
                        stream_name,
                        success: false,
                        error_message: Some(format!("No message received within {}s", message_timeout.as_secs())),
                        connection_time_ms: connection_time.as_millis() as u64,
                        time_to_first_message_ms: None,
                        first_message_sample: None,
                    }
                }
                Ok(None) => {
                    error!("  ✗ {} | {} | {}", stream.exchange, stream.market_type, stream.stream_type);
                    error!("    → Stream closed without message");
                    StreamTestResult {
                        stream_name,
                        success: false,
                        error_message: Some("Stream closed without message".to_string()),
                        connection_time_ms: connection_time.as_millis() as u64,
                        time_to_first_message_ms: None,
                        first_message_sample: None,
                    }
                }
                Ok(Some(Err(e))) => {
                    error!("  ✗ {} | {} | {}", stream.exchange, stream.market_type, stream.stream_type);
                    error!("    → WebSocket error: {}", e);
                    StreamTestResult {
                        stream_name,
                        success: false,
                        error_message: Some(format!("WebSocket error: {}", e)),
                        connection_time_ms: connection_time.as_millis() as u64,
                        time_to_first_message_ms: None,
                        first_message_sample: None,
                    }
                }
                Ok(Some(Ok(Message::Text(text)))) => {
                    let text_str = text.to_string();
                    let time_to_first = message_start.elapsed().as_millis() as u64;

                    // Try to get next message if this one looks like a subscription confirmation
                    let mut final_sample = text_str.clone();
                    if text_str.contains("subscribe") || text_str.contains("result") || text_str.contains("success") {
                        // Wait for next message (actual data)
                        if let Ok(Some(Ok(Message::Text(data_msg)))) =
                            timeout(Duration::from_secs(3), msg_stream.next()).await
                        {
                            final_sample = data_msg.to_string();
                        }
                    }

                    let sample = final_sample.chars().take(300).collect::<String>();
                    info!("  ✓ {} | {} | {}", stream.exchange, stream.market_type, stream.stream_type);
                    info!(
                        "    → Connected: {}ms | First msg: {}ms",
                        connection_time.as_millis(),
                        time_to_first
                    );
                    info!("    → Sample: {}", sample);
                    StreamTestResult {
                        stream_name,
                        success: true,
                        error_message: None,
                        connection_time_ms: connection_time.as_millis() as u64,
                        time_to_first_message_ms: Some(time_to_first),
                        first_message_sample: Some(final_sample), // Store full message
                    }
                }
                Ok(Some(Ok(_))) => {
                    error!("  ✗ {} | {} | {}", stream.exchange, stream.market_type, stream.stream_type);
                    error!("    → Unexpected message type (binary)");
                    StreamTestResult {
                        stream_name,
                        success: false,
                        error_message: Some("Unexpected message type (binary)".to_string()),
                        connection_time_ms: connection_time.as_millis() as u64,
                        time_to_first_message_ms: None,
                        first_message_sample: None,
                    }
                }
            }
        }
    }
}

#[tokio::test]
#[ignore] // Run with: cargo test --test market_stream_validation -- --ignored
async fn test_market_streams_connectivity() {
    arkin_core::prelude::init_tracing();

    let config = StreamValidationConfig::new();
    let message_timeout = Duration::from_secs(config.message_timeout_secs);

    info!("╔════════════════════════════════════════════════════════════════╗");
    info!(
        "║   Market Stream Endpoint Validation Test ({}s timeout)   ║",
        config.message_timeout_secs
    );
    info!(
        "║            Testing {} stream endpoints                   ║",
        config.test_streams.len()
    );
    info!("╚════════════════════════════════════════════════════════════════╝");

    let mut results = vec![];
    let mut successful = 0;
    let mut failed = 0;

    for stream in &config.test_streams {
        info!("Testing: {}", stream.display_name());
        let result = test_stream(stream, message_timeout).await;

        if result.success {
            successful += 1;
            info!(
                "  ✓ SUCCESS - connection: {}ms, first message: {}ms",
                result.connection_time_ms,
                result.time_to_first_message_ms.unwrap_or(0)
            );
            if let Some(sample) = &result.first_message_sample {
                info!("    Sample: {}", sample);
            }
        } else {
            failed += 1;
            error!(
                "  ✗ FAILED - {}",
                result.error_message.as_ref().unwrap_or(&"Unknown error".to_string())
            );
        }

        results.push(result);
    }

    // Print summary
    info!("\n╔════════════════════════════════════════════════════════════════╗");
    info!("║                    TEST SUMMARY                                ║");
    info!("╚════════════════════════════════════════════════════════════════╝");
    info!(
        "Total:      {}\nSuccessful: {}\nFailed:     {}\nSuccess %:  {:.1}%",
        results.len(),
        successful,
        failed,
        (successful as f64 / results.len() as f64) * 100.0
    );

    // Group by exchange
    info!("\n┌─ BY EXCHANGE ─────────────────────────────────────────────────┐");
    for exchange_filter in [Exchange::Binance, Exchange::Okx, Exchange::Bybit, Exchange::Coinbase] {
        let exchange_results: Vec<_> = results
            .iter()
            .filter(|r| r.stream_name.starts_with(&exchange_filter.to_string()))
            .collect();

        if !exchange_results.is_empty() {
            let exchange_successful = exchange_results.iter().filter(|r| r.success).count();
            info!("  {} - {}/{} ✓", exchange_filter, exchange_successful, exchange_results.len());
            for result in &exchange_results {
                if !result.success {
                    info!(
                        "    ✗ {} - {}",
                        result.stream_name,
                        result.error_message.as_ref().unwrap_or(&"Unknown".to_string())
                    );
                }
            }
        }
    }

    info!("└────────────────────────────────────────────────────────────────┘\n");

    // Assert at least 50% success rate
    let success_rate = successful as f64 / results.len() as f64;
    assert!(
        success_rate >= 0.5,
        "Market stream connectivity test failed: only {:.1}% endpoints responded",
        success_rate * 100.0
    );

    // Save successful message samples for parser development
    info!("\n╔════════════════════════════════════════════════════════════════╗");
    info!("║            Saving Message Samples for Parser Development       ║");
    info!("╚════════════════════════════════════════════════════════════════╝\n");

    use std::fs;
    use std::path::Path;

    let samples_dir = "message_samples";
    if !Path::new(samples_dir).exists() {
        fs::create_dir_all(samples_dir).ok();
    }

    for result in &results {
        if result.success {
            if let Some(sample) = &result.first_message_sample {
                // Clean up stream name for filename
                let filename = result.stream_name.replace(" ", "_").to_lowercase();
                let filepath = format!("{}/{}.json", samples_dir, filename);

                match fs::write(&filepath, sample) {
                    Ok(_) => info!("  ✓ Saved: {}", filepath),
                    Err(e) => error!("  ✗ Failed to save {}: {}", filepath, e),
                }
            }
        }
    }

    info!("\nSamples saved to: {}/", samples_dir);
    info!("Use these samples to implement parsers for each exchange/stream type\n");
}
