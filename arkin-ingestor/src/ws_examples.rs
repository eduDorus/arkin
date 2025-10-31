//! Examples demonstrating the Binance WebSocket client
//!
//! Run with: cargo run --example binance_ws_examples

use serde_json::Value;
use tracing::info;

/// Example 1: Basic streaming of BTCUSDT trades
pub async fn example_basic_streaming() -> Result<(), String> {
    info!("=== Example 1: Basic Streaming ===\n");

    // In practice, you would:
    // 1. Create a config
    // 2. Create a client
    // 3. Spawn it
    // 4. Process messages

    info!("Configuration: BTCUSDT aggTrade stream");
    info!("- URL: wss://fstream.binance.com/ws");
    info!("- Subscribe: btcusdt@aggTrade");
    info!("- Update frequency: 100ms");
    info!("\nExpected output:");
    info!("{{ \"e\": \"aggTrade\", \"s\": \"BTCUSDT\", \"p\": \"42500.50\", \"q\": \"0.5\", ... }}");

    Ok(())
}

/// Example 2: Monitoring multiple symbol streams
pub async fn example_multiple_streams() -> Result<(), String> {
    info!("\n=== Example 2: Multiple Streams ===\n");

    let streams = vec!["btcusdt@aggTrade", "ethusdt@aggTrade", "solusdt@aggTrade"];

    info!("Subscribing to {} streams:", streams.len());
    for stream in &streams {
        info!("  - {}", stream);
    }

    info!("\nSubscription message:");
    info!("{{");
    info!("  \"method\": \"SUBSCRIBE\",");
    info!("  \"params\": [\"btcusdt@aggTrade\", \"ethusdt@aggTrade\", \"solusdt@aggTrade\"],");
    info!("  \"id\": 1");
    info!("}}");

    Ok(())
}

/// Example 3: Handling reconnection logic
pub async fn example_reconnection() -> Result<(), String> {
    info!("\n=== Example 3: Reconnection Logic ===\n");

    info!("Reconnection Strategy:");
    info!("  Max Attempts: 10");
    info!("  Initial Backoff: 1s");
    info!("  Backoff Growth: Exponential (2x)");
    info!("\nBackoff Sequence:");
    for attempt in 1..=6 {
        let backoff = 1000 * 2u64.pow(attempt - 1);
        info!("  Attempt {}: {} ms ({:.1}s)", attempt, backoff, backoff as f64 / 1000.0);
    }

    Ok(())
}

/// Example 4: Parsing trade data
pub async fn example_parse_trade() -> Result<(), String> {
    info!("\n=== Example 4: Parsing Trade Data ===\n");

    let trade_json = r#"{
  "e": "aggTrade",
  "E": 1695369600123,
  "s": "BTCUSDT",
  "a": 123456789,
  "p": "27500.50",
  "q": "1.5",
  "f": 100,
  "l": 105,
  "T": 1695369600000,
  "m": false
}"#;

    info!("Raw JSON:");
    info!("{}", trade_json);

    if let Ok(json) = serde_json::from_str::<Value>(trade_json) {
        info!("\nParsed Fields:");
        info!("  Event Type: {}", json["e"].as_str().unwrap_or("N/A"));
        info!("  Symbol: {}", json["s"].as_str().unwrap_or("N/A"));
        info!("  Price: {}", json["p"].as_str().unwrap_or("N/A"));
        info!("  Quantity: {}", json["q"].as_str().unwrap_or("N/A"));
        info!("  Buyer is Maker: {}", json["m"].as_bool().unwrap_or(false));
    }

    Ok(())
}

/// Example 5: Keepalive ping/pong mechanism
pub async fn example_keepalive() -> Result<(), String> {
    info!("\n=== Example 5: Keepalive Mechanism ===\n");

    info!("Binance Ping/Pong Requirements:");
    info!("  Server sends PING: Every 3 minutes");
    info!("  Client sends PONG: Within 10 minutes");
    info!("  Connection timeout: No message for 10 minutes → disconnect");
    info!("\nClient Implementation:");
    info!("  1. Receive PING frame from server");
    info!("  2. Respond with PONG frame immediately");
    info!("  3. Timeout after 10 minutes of inactivity");
    info!("  4. Reconnect if pong timeout occurs");
    info!("\nTiming Configuration:");
    info!("  - ping_interval_secs: 180 (server sends every 3 min)");
    info!("  - pong_timeout_secs: 600 (disconnect after 10 min)");

    Ok(())
}

/// Example 6: Subscription and unsubscription
pub async fn example_subscription_lifecycle() -> Result<(), String> {
    info!("\n=== Example 6: Subscription Lifecycle ===\n");

    info!("1. Subscribe to streams:");
    info!("Request:");
    info!("{{");
    info!("  \"method\": \"SUBSCRIBE\",");
    info!("  \"params\": [\"btcusdt@aggTrade\", \"ethusdt@aggTrade\"],");
    info!("  \"id\": 1");
    info!("}}");
    info!("/n");
    info!("Response:");
    info!("{{");
    info!("  \"result\": null,");
    info!("  \"id\": 1");
    info!("}}");

    info!("\n2. List subscriptions:");
    info!("Request:");
    info!("{{");
    info!("  \"method\": \"LIST_SUBSCRIPTIONS\",");
    info!("  \"id\": 2");
    info!("}}");
    info!("/n");
    info!("Response:");
    info!("{{");
    info!("  \"result\": [\"btcusdt@aggTrade\", \"ethusdt@aggTrade\"],");
    info!("  \"id\": 2");
    info!("}}");

    info!("\n3. Unsubscribe from stream:");
    info!("Request:");
    info!("{{");
    info!("  \"method\": \"UNSUBSCRIBE\",");
    info!("  \"params\": [\"btcusdt@aggTrade\"],");
    info!("  \"id\": 3");
    info!("}}");
    info!("/n");
    info!("Response:");
    info!("{{");
    info!("  \"result\": null,");
    info!("  \"id\": 3");
    info!("}}");

    Ok(())
}

/// Example 7: Error scenarios
pub async fn example_error_scenarios() -> Result<(), String> {
    info!("\n=== Example 7: Error Scenarios ===\n");

    info!("Scenario 1: Network Connection Failure");
    info!("  Error: \"Connection refused\"");
    info!("  Recovery: Exponential backoff reconnection");
    info!("  Result: Automatically reconnects with state preserved");

    info!("\nScenario 2: Invalid Subscription");
    info!("  Error: {{\"code\": 2, \"msg\": \"Invalid request: unknown method\"}}");
    info!("  Recovery: Log error and continue");
    info!("  Result: Connection remains open, other streams unaffected");

    info!("\nScenario 3: Pong Timeout");
    info!("  Error: \"Pong timeout - no message received\"");
    info!("  Recovery: Trigger reconnection");
    info!("  Result: New connection with fresh stream subscriptions");

    info!("\nScenario 4: Message Parse Error");
    info!("  Error: \"Failed to parse message\"");
    info!("  Recovery: Log error and continue");
    info!("  Result: Invalid message ignored, connection continues");

    Ok(())
}

/// Example 8: Production configuration
pub async fn example_production_config() -> Result<(), String> {
    info!("\n=== Example 8: Production Configuration ===\n");

    info!("let config = WsConfig {{");
    info!("    url: \"wss://fstream.binance.com/ws\".to_string(),");
    info!("    streams: vec![");
    info!("        \"btcusdt@aggTrade\".to_string(),");
    info!("        \"ethusdt@aggTrade\".to_string(),");
    info!("        \"solusdt@aggTrade\".to_string(),");
    info!("    ],");
    info!("    reconnect_backoff_ms: 1000,      // Start with 1s");
    info!("    max_reconnect_attempts: 10,      // Try 10 times");
    info!("    ping_interval_secs: 180,         // Check every 3 min");
    info!("    pong_timeout_secs: 600,          // 10 min timeout");
    info!("}};");
    info!("/n");
    info!("let (mut client, mut rx) = BinanceWsClient::new(config);");
    info!("/n");
    info!("// Spawn in background");
    info!("tokio::spawn(async move {{");
    info!("    if let Err(e) = client.run().await {{");
    info!("        einfo!(\"Fatal error: {{}}\", e);");
    info!("    }}");
    info!("}});");
    info!("/n");
    info!("// Process messages with backpressure handling");
    info!("while let Some(msg) = rx.recv().await {{");
    info!("    // Your business logic here");
    info!("}}");

    Ok(())
}

/// Example 9: Error Statistics Monitoring
pub async fn example_error_statistics() -> Result<(), String> {
    info!("\n=== Example 9: Error Statistics Monitoring ===\n");

    info!("// Access error statistics from the client:");
    info!("let client = /* ... WsClient instance ... */;");
    info!("/n");
    info!("// Get current error statistics");
    info!("let stats = client.error_stats();");
    info!("info!(\"Error Statistics: {{}}\", stats);");
    info!("/n");
    info!("// Example output:");
    info!("// Error Statistics: Total Errors: 5 | Breakdown: connection_failed: 2, websocket_error: 1, stale_connection: 2");
    info!("/n");
    info!("// Error Categories Tracked:");
    info!("//   - ConnectionFailed: Failed to establish connection");
    info!("//   - WebSocketError: Protocol-level WebSocket errors");
    info!("//   - ParseError: Failed to parse JSON messages");
    info!("//   - PongSendFailed: Failed to send pong response");
    info!("//   - PingSendFailed: Failed to send ping request");
    info!("//   - SubscriptionFailed: Failed to subscribe to stream");
    info!("//   - StaleConnection: No messages received for timeout period");
    info!("//   - UnexpectedBinary: Received unexpected binary message");
    info!("/n");
    info!("// Usage in monitoring/alerting:");
    info!("if stats.total_errors > 100 {{");
    info!("    // Alert: Too many errors");
    info!("    alert(format!(\"High error count detected: {{}}\", stats));");
    info!("}}");
    info!("/n");
    info!("// Check for specific error patterns:");
    info!("let stale_connections = stats");
    info!("    .errors_by_type");
    info!("    .iter()");
    info!("    .find(|(name, _)| name == \"stale_connection\")");
    info!("    .map(|(_, count)| count)");
    info!("    .unwrap_or(&0);");
    info!("/n");
    info!("if *stale_connections > 5 {{");
    info!("    // Alert: Connection stability issue");
    info!("    alert(\"Stale connection issues detected\");");
    info!("}}");

    Ok(())
}
