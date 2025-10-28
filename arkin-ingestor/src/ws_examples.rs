//! Examples demonstrating the Binance WebSocket client
//!
//! Run with: cargo run --example binance_ws_examples

use serde_json::Value;

/// Example 1: Basic streaming of BTCUSDT trades
pub async fn example_basic_streaming() -> Result<(), String> {
    println!("=== Example 1: Basic Streaming ===\n");

    // In practice, you would:
    // 1. Create a config
    // 2. Create a client
    // 3. Spawn it
    // 4. Process messages

    println!("Configuration: BTCUSDT aggTrade stream");
    println!("- URL: wss://fstream.binance.com/ws");
    println!("- Subscribe: btcusdt@aggTrade");
    println!("- Update frequency: 100ms");
    println!("\nExpected output:");
    println!("{{ \"e\": \"aggTrade\", \"s\": \"BTCUSDT\", \"p\": \"42500.50\", \"q\": \"0.5\", ... }}");

    Ok(())
}

/// Example 2: Monitoring multiple symbol streams
pub async fn example_multiple_streams() -> Result<(), String> {
    println!("\n=== Example 2: Multiple Streams ===\n");

    let streams = vec!["btcusdt@aggTrade", "ethusdt@aggTrade", "solusdt@aggTrade"];

    println!("Subscribing to {} streams:", streams.len());
    for stream in &streams {
        println!("  - {}", stream);
    }

    println!("\nSubscription message:");
    println!("{{");
    println!("  \"method\": \"SUBSCRIBE\",");
    println!("  \"params\": [\"btcusdt@aggTrade\", \"ethusdt@aggTrade\", \"solusdt@aggTrade\"],");
    println!("  \"id\": 1");
    println!("}}");

    Ok(())
}

/// Example 3: Handling reconnection logic
pub async fn example_reconnection() -> Result<(), String> {
    println!("\n=== Example 3: Reconnection Logic ===\n");

    println!("Reconnection Strategy:");
    println!("  Max Attempts: 10");
    println!("  Initial Backoff: 1s");
    println!("  Backoff Growth: Exponential (2x)");
    println!("\nBackoff Sequence:");
    for attempt in 1..=6 {
        let backoff = 1000 * 2u64.pow(attempt - 1);
        println!("  Attempt {}: {} ms ({:.1}s)", attempt, backoff, backoff as f64 / 1000.0);
    }

    Ok(())
}

/// Example 4: Parsing trade data
pub async fn example_parse_trade() -> Result<(), String> {
    println!("\n=== Example 4: Parsing Trade Data ===\n");

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

    println!("Raw JSON:");
    println!("{}", trade_json);

    if let Ok(json) = serde_json::from_str::<Value>(trade_json) {
        println!("\nParsed Fields:");
        println!("  Event Type: {}", json["e"].as_str().unwrap_or("N/A"));
        println!("  Symbol: {}", json["s"].as_str().unwrap_or("N/A"));
        println!("  Price: {}", json["p"].as_str().unwrap_or("N/A"));
        println!("  Quantity: {}", json["q"].as_str().unwrap_or("N/A"));
        println!("  Buyer is Maker: {}", json["m"].as_bool().unwrap_or(false));
    }

    Ok(())
}

/// Example 5: Keepalive ping/pong mechanism
pub async fn example_keepalive() -> Result<(), String> {
    println!("\n=== Example 5: Keepalive Mechanism ===\n");

    println!("Binance Ping/Pong Requirements:");
    println!("  Server sends PING: Every 3 minutes");
    println!("  Client sends PONG: Within 10 minutes");
    println!("  Connection timeout: No message for 10 minutes → disconnect");
    println!("\nClient Implementation:");
    println!("  1. Receive PING frame from server");
    println!("  2. Respond with PONG frame immediately");
    println!("  3. Timeout after 10 minutes of inactivity");
    println!("  4. Reconnect if pong timeout occurs");
    println!("\nTiming Configuration:");
    println!("  - ping_interval_secs: 180 (server sends every 3 min)");
    println!("  - pong_timeout_secs: 600 (disconnect after 10 min)");

    Ok(())
}

/// Example 6: Subscription and unsubscription
pub async fn example_subscription_lifecycle() -> Result<(), String> {
    println!("\n=== Example 6: Subscription Lifecycle ===\n");

    println!("1. Subscribe to streams:");
    println!("Request:");
    println!("{{");
    println!("  \"method\": \"SUBSCRIBE\",");
    println!("  \"params\": [\"btcusdt@aggTrade\", \"ethusdt@aggTrade\"],");
    println!("  \"id\": 1");
    println!("}}");
    println!();
    println!("Response:");
    println!("{{");
    println!("  \"result\": null,");
    println!("  \"id\": 1");
    println!("}}");

    println!("\n2. List subscriptions:");
    println!("Request:");
    println!("{{");
    println!("  \"method\": \"LIST_SUBSCRIPTIONS\",");
    println!("  \"id\": 2");
    println!("}}");
    println!();
    println!("Response:");
    println!("{{");
    println!("  \"result\": [\"btcusdt@aggTrade\", \"ethusdt@aggTrade\"],");
    println!("  \"id\": 2");
    println!("}}");

    println!("\n3. Unsubscribe from stream:");
    println!("Request:");
    println!("{{");
    println!("  \"method\": \"UNSUBSCRIBE\",");
    println!("  \"params\": [\"btcusdt@aggTrade\"],");
    println!("  \"id\": 3");
    println!("}}");
    println!();
    println!("Response:");
    println!("{{");
    println!("  \"result\": null,");
    println!("  \"id\": 3");
    println!("}}");

    Ok(())
}

/// Example 7: Error scenarios
pub async fn example_error_scenarios() -> Result<(), String> {
    println!("\n=== Example 7: Error Scenarios ===\n");

    println!("Scenario 1: Network Connection Failure");
    println!("  Error: \"Connection refused\"");
    println!("  Recovery: Exponential backoff reconnection");
    println!("  Result: Automatically reconnects with state preserved");

    println!("\nScenario 2: Invalid Subscription");
    println!("  Error: {{\"code\": 2, \"msg\": \"Invalid request: unknown method\"}}");
    println!("  Recovery: Log error and continue");
    println!("  Result: Connection remains open, other streams unaffected");

    println!("\nScenario 3: Pong Timeout");
    println!("  Error: \"Pong timeout - no message received\"");
    println!("  Recovery: Trigger reconnection");
    println!("  Result: New connection with fresh stream subscriptions");

    println!("\nScenario 4: Message Parse Error");
    println!("  Error: \"Failed to parse message\"");
    println!("  Recovery: Log error and continue");
    println!("  Result: Invalid message ignored, connection continues");

    Ok(())
}

/// Example 8: Production configuration
pub async fn example_production_config() -> Result<(), String> {
    println!("\n=== Example 8: Production Configuration ===\n");

    println!("let config = WsConfig {{");
    println!("    url: \"wss://fstream.binance.com/ws\".to_string(),");
    println!("    streams: vec![");
    println!("        \"btcusdt@aggTrade\".to_string(),");
    println!("        \"ethusdt@aggTrade\".to_string(),");
    println!("        \"solusdt@aggTrade\".to_string(),");
    println!("    ],");
    println!("    reconnect_backoff_ms: 1000,      // Start with 1s");
    println!("    max_reconnect_attempts: 10,      // Try 10 times");
    println!("    ping_interval_secs: 180,         // Check every 3 min");
    println!("    pong_timeout_secs: 600,          // 10 min timeout");
    println!("}};");
    println!();
    println!("let (mut client, mut rx) = BinanceWsClient::new(config);");
    println!();
    println!("// Spawn in background");
    println!("tokio::spawn(async move {{");
    println!("    if let Err(e) = client.run().await {{");
    println!("        eprintln!(\"Fatal error: {{}}\", e);");
    println!("    }}");
    println!("}});");
    println!();
    println!("// Process messages with backpressure handling");
    println!("while let Some(msg) = rx.recv().await {{");
    println!("    // Your business logic here");
    println!("}}");

    Ok(())
}

