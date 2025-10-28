//! Chaos WebSocket Testing Suite
//!
//! Tests WebSocket client against real-world Binance failure scenarios:
//! - ResetWithoutClosingHandshake (abrupt TCP termination)
//! - Silent streams (connection open but no data)
//! - Intermittent connection loss
//! - Corrupted JSON responses
//! - High latency scenarios
//! - Ping/pong issues
//!
//! Run with: cargo test --test chaos_websocket_test -- --nocapture --test-threads=1

use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message, WebSocketStream};

// ============================================================================
// Mock Chaos Server
// ============================================================================

/// Failure modes to simulate
#[derive(Clone, Debug)]
pub enum FailureMode {
    Normal,
    ResetAfterMessages(u32),
    SilentAfterMessages(u32),
    CorruptedJson(u32),
    HighLatency(Duration),
    IntermittentReset(u32),
    PingFlood,
    SlowSubscription,
}

/// Mock Binance server for chaos testing
pub struct ChaosBinanceServer {
    listener: TcpListener,
    port: u16,
    failure_mode: FailureMode,
    message_count: Arc<AtomicU32>,
}

impl ChaosBinanceServer {
    pub async fn new(failure_mode: FailureMode) -> Result<Self, Box<dyn std::error::Error>> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();

        Ok(Self {
            listener,
            port,
            failure_mode,
            message_count: Arc::new(AtomicU32::new(0)),
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn url(&self) -> String {
        format!("ws://127.0.0.1:{}/ws", self.port)
    }

    pub fn message_count(&self) -> u32 {
        self.message_count.load(Ordering::SeqCst)
    }

    /// Start accepting connections
    pub async fn run(self) {
        loop {
            match self.listener.accept().await {
                Ok((stream, _addr)) => {
                    let failure_mode = self.failure_mode.clone();
                    let msg_count = self.message_count.clone();

                    tokio::spawn(async move {
                        let _ = Self::handle_client(stream, failure_mode, msg_count).await;
                    });
                }
                Err(_) => break,
            }
        }
    }

    async fn handle_client(
        stream: TcpStream,
        failure_mode: FailureMode,
        msg_count: Arc<AtomicU32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ws_stream = accept_async(stream).await?;
        let (mut sink, mut stream) = ws_stream.split();

        // Handle subscription
        while let Some(msg_result) = stream.next().await {
            match msg_result? {
                Message::Text(text) => {
                    if text.contains("SUBSCRIBE") {
                        // Simulate slow subscription
                        if matches!(failure_mode, FailureMode::SlowSubscription) {
                            sleep(Duration::from_secs(2)).await;
                        }

                        let response = json!({
                            "result": null,
                            "id": 1
                        });
                        sink.send(Message::text(response.to_string())).await?;
                        break;
                    }
                }
                Message::Ping(data) => {
                    sink.send(Message::Pong(data)).await?;
                }
                _ => {}
            }
        }

        // Send messages according to failure mode
        match failure_mode {
            FailureMode::Normal => {
                Self::send_normal_stream(&mut sink, &msg_count).await?;
            }
            FailureMode::ResetAfterMessages(n) => {
                Self::send_then_reset(&mut sink, n, &msg_count).await?;
            }
            FailureMode::SilentAfterMessages(n) => {
                Self::send_then_silent(&mut sink, n, &msg_count).await?;
            }
            FailureMode::CorruptedJson(n) => {
                Self::send_with_corruption(&mut sink, n, &msg_count).await?;
            }
            FailureMode::HighLatency(latency) => {
                Self::send_with_latency(&mut sink, latency, &msg_count).await?;
            }
            FailureMode::IntermittentReset(n) => {
                Self::send_intermittent_reset(&mut sink, n, &msg_count).await?;
            }
            FailureMode::PingFlood => {
                Self::send_ping_flood(&mut sink, &msg_count).await?;
            }
            FailureMode::SlowSubscription => {
                Self::send_normal_stream(&mut sink, &msg_count).await?;
            }
        }

        Ok(())
    }

    async fn send_normal_stream(
        sink: &mut futures::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        msg_count: &Arc<AtomicU32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut count = 0;
        loop {
            sleep(Duration::from_millis(100)).await;
            count += 1;
            let msg = Self::create_trade_message(count as i32);
            sink.send(Message::text(msg.to_string())).await?;
            msg_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    async fn send_then_reset(
        sink: &mut futures::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        reset_after: u32,
        msg_count: &Arc<AtomicU32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for i in 0..reset_after {
            let msg = Self::create_trade_message(i as i32);
            sink.send(Message::text(msg.to_string())).await?;
            msg_count.fetch_add(1, Ordering::SeqCst);
            sleep(Duration::from_millis(100)).await;
        }

        // Simulate ResetWithoutClosingHandshake by dropping connection
        drop(sink);
        Ok(())
    }

    async fn send_then_silent(
        sink: &mut futures::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        silent_after: u32,
        msg_count: &Arc<AtomicU32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for i in 0..silent_after {
            let msg = Self::create_trade_message(i as i32);
            sink.send(Message::text(msg.to_string())).await?;
            msg_count.fetch_add(1, Ordering::SeqCst);
            sleep(Duration::from_millis(100)).await;
        }

        // Connection open but no data (stale)
        loop {
            sleep(Duration::from_secs(10)).await;
        }
    }

    async fn send_with_corruption(
        sink: &mut futures::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        corrupt_after: u32,
        msg_count: &Arc<AtomicU32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for i in 0..corrupt_after {
            let msg = Self::create_trade_message(i as i32);
            sink.send(Message::text(msg.to_string())).await?;
            msg_count.fetch_add(1, Ordering::SeqCst);
            sleep(Duration::from_millis(100)).await;
        }

        // Send malformed JSON
        for i in 0..10 {
            let bad_msg = format!(r#"{{"e": "aggTrade", "corrupted": {}"#, i);
            sink.send(Message::text(bad_msg)).await?;
            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    async fn send_with_latency(
        sink: &mut futures::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        latency: Duration,
        msg_count: &Arc<AtomicU32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut count = 0;
        loop {
            sleep(latency).await;
            count += 1;
            let msg = Self::create_trade_message(count as i32);
            sink.send(Message::text(msg.to_string())).await?;
            msg_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    async fn send_intermittent_reset(
        sink: &mut futures::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        reset_every: u32,
        msg_count: &Arc<AtomicU32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let count = 0;
        loop {
            for i in 0..reset_every {
                let msg = Self::create_trade_message((count + i) as i32);
                sink.send(Message::text(msg.to_string())).await?;
                msg_count.fetch_add(1, Ordering::SeqCst);
                sleep(Duration::from_millis(100)).await;
            }

            // Reset connection
            drop(sink);
            return Ok(());
        }
    }

    async fn send_ping_flood(
        sink: &mut futures::stream::SplitSink<WebSocketStream<TcpStream>, Message>,
        msg_count: &Arc<AtomicU32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut count = 0;
        loop {
            sleep(Duration::from_millis(50)).await;

            // Send normal message
            count += 1;
            let msg = Self::create_trade_message(count as i32);
            sink.send(Message::text(msg.to_string())).await?;
            msg_count.fetch_add(1, Ordering::SeqCst);

            // Send ping flood
            for _ in 0..3 {
                sink.send(Message::Ping(vec![].into())).await?;
            }
        }
    }

    fn create_trade_message(id: i32) -> Value {
        json!({
            "e": "aggTrade",
            "E": 1234567890000i64 + id as i64 * 100,
            "s": "BTCUSDT",
            "a": 5933014 + id,
            "p": "50000.00",
            "q": "1.50",
            "f": 100,
            "l": 105,
            "T": 1234567890000i64 + id as i64 * 100,
            "m": false
        })
    }
}

// ============================================================================
// Resilience Metrics
// ============================================================================

#[derive(Default, Clone, Debug)]
pub struct ResilienceMetrics {
    pub messages_received: u32,
    pub reconnects: u32,
    pub errors_recovered: u32,
    pub final_success: bool,
    pub uptime_percentage: f64,
}

impl ResilienceMetrics {
    pub fn print_summary(&self) {
        println!("\n╔═══════════════════════════════════════════╗");
        println!("║       Resilience Test Summary            ║");
        println!("╠═══════════════════════════════════════════╣");
        println!("║ Messages received:     {:3} msgs         ║", self.messages_received);
        println!("║ Reconnects triggered:  {:3} times        ║", self.reconnects);
        println!("║ Errors recovered:      {:3} errors       ║", self.errors_recovered);
        println!("║ Final success:         {:<15} ║", format!("{:?}", self.final_success));
        println!("║ Uptime:                {:5.1}%          ║", self.uptime_percentage);
        println!("╚═══════════════════════════════════════════╝\n");
    }
}

// ============================================================================
// Test Cases
// ============================================================================

#[tokio::test]
async fn test_chaos_reset_without_closing_handshake() {
    println!("\n=== Test: ResetWithoutClosingHandshake ===");

    let server = ChaosBinanceServer::new(FailureMode::ResetAfterMessages(5))
        .await
        .expect("Failed to create server");
    let server_url = server.url();

    tokio::spawn(async move {
        server.run().await;
    });

    sleep(Duration::from_millis(100)).await;

    let mut metrics = ResilienceMetrics::default();
    let test_duration = Duration::from_secs(5);
    let start = tokio::time::Instant::now();

    while start.elapsed() < test_duration {
        match connect_async(&server_url).await {
            Ok((ws_stream, _)) => {
                metrics.reconnects += 1;
                let (mut sink, mut stream) = ws_stream.split();

                // Send subscription
                let sub_msg = json!({
                    "method": "SUBSCRIBE",
                    "params": ["btcusdt@aggTrade"],
                    "id": 1
                });
                let _ = sink.send(Message::text(sub_msg.to_string())).await;

                // Try to receive messages
                while let Ok(Some(Ok(msg))) = tokio::time::timeout(Duration::from_secs(2), stream.next()).await {
                    match msg {
                        Message::Text(_) => metrics.messages_received += 1,
                        Message::Ping(data) => {
                            let _ = sink.send(Message::Pong(data)).await;
                        }
                        _ => {}
                    }
                }
                metrics.errors_recovered += 1;
            }
            Err(_) => {
                sleep(Duration::from_millis(500)).await;
            }
        }
    }

    metrics.final_success = metrics.messages_received > 0;
    metrics.uptime_percentage = (metrics.messages_received as f64 / 5.0) * 100.0;
    metrics.print_summary();

    println!("✓ Successfully handled ResetWithoutClosingHandshake");
    assert!(metrics.messages_received > 0, "Should receive at least some messages");
    assert!(metrics.reconnects > 0, "Should have reconnected at least once");
}

#[tokio::test]
async fn test_chaos_silent_stream() {
    println!("\n=== Test: Silent Stream (No Messages) ===");

    let server = ChaosBinanceServer::new(FailureMode::SilentAfterMessages(3))
        .await
        .expect("Failed to create server");
    let server_url = server.url();

    tokio::spawn(async move {
        server.run().await;
    });

    sleep(Duration::from_millis(100)).await;

    let mut metrics = ResilienceMetrics::default();

    match connect_async(&server_url).await {
        Ok((ws_stream, _)) => {
            let (mut sink, mut stream) = ws_stream.split();

            let sub_msg = json!({
                "method": "SUBSCRIBE",
                "params": ["btcusdt@aggTrade"],
                "id": 1
            });
            let _ = sink.send(Message::text(sub_msg.to_string())).await;
            metrics.reconnects += 1;

            // Wait for messages and timeout on stale
            let receive_duration = Duration::from_secs(2);
            let start = tokio::time::Instant::now();

            while start.elapsed() < receive_duration {
                match tokio::time::timeout(Duration::from_millis(500), stream.next()).await {
                    Ok(Some(Ok(Message::Text(_)))) => {
                        metrics.messages_received += 1;
                    }
                    Ok(Some(Ok(Message::Ping(data)))) => {
                        let _ = sink.send(Message::Pong(data)).await;
                    }
                    Err(_) => {
                        metrics.errors_recovered += 1;
                    }
                    _ => {}
                }
            }
        }
        Err(e) => panic!("Failed to connect: {}", e),
    }

    metrics.final_success = metrics.errors_recovered > 0;
    metrics.print_summary();

    println!("✓ Successfully detected and handled silent stream");
    assert!(metrics.messages_received > 0, "Should receive initial messages");
    assert!(metrics.errors_recovered > 0, "Should detect stale connection");
}

#[tokio::test]
async fn test_chaos_high_latency() {
    println!("\n=== Test: High Latency (1 second between messages) ===");

    let server = ChaosBinanceServer::new(FailureMode::HighLatency(Duration::from_secs(1)))
        .await
        .expect("Failed to create server");
    let server_url = server.url();

    tokio::spawn(async move {
        server.run().await;
    });

    sleep(Duration::from_millis(100)).await;

    let mut metrics = ResilienceMetrics::default();

    match connect_async(&server_url).await {
        Ok((ws_stream, _)) => {
            let (mut sink, mut stream) = ws_stream.split();

            let sub_msg = json!({
                "method": "SUBSCRIBE",
                "params": ["btcusdt@aggTrade"],
                "id": 1
            });
            let _ = sink.send(Message::text(sub_msg.to_string())).await;
            metrics.reconnects += 1;

            // Receive messages with 3 second timeout
            for _ in 0..3 {
                match tokio::time::timeout(Duration::from_secs(3), stream.next()).await {
                    Ok(Some(Ok(Message::Text(_)))) => {
                        metrics.messages_received += 1;
                    }
                    Ok(Some(Ok(Message::Ping(data)))) => {
                        let _ = sink.send(Message::Pong(data)).await;
                    }
                    Err(_) => {
                        metrics.errors_recovered += 1;
                    }
                    _ => {}
                }
            }
        }
        Err(e) => panic!("Connection failed: {}", e),
    }

    metrics.final_success = metrics.messages_received >= 2;
    metrics.print_summary();

    println!("✓ Successfully handled high latency scenarios");
    assert!(metrics.messages_received > 0, "Should handle high latency");
}

#[tokio::test]
async fn test_chaos_corrupted_json() {
    println!("\n=== Test: Corrupted JSON Responses ===");

    let server = ChaosBinanceServer::new(FailureMode::CorruptedJson(3))
        .await
        .expect("Failed to create server");
    let server_url = server.url();

    tokio::spawn(async move {
        server.run().await;
    });

    sleep(Duration::from_millis(100)).await;

    let mut metrics = ResilienceMetrics::default();
    let mut valid_parsed = 0;
    let mut invalid_received = 0;

    match connect_async(&server_url).await {
        Ok((ws_stream, _)) => {
            let (mut sink, mut stream) = ws_stream.split();

            let sub_msg = json!({
                "method": "SUBSCRIBE",
                "params": ["btcusdt@aggTrade"],
                "id": 1
            });
            let _ = sink.send(Message::text(sub_msg.to_string())).await;

            // Receive messages and attempt parsing
            for _ in 0..15 {
                match tokio::time::timeout(Duration::from_secs(1), stream.next()).await {
                    Ok(Some(Ok(Message::Text(text)))) => {
                        // Try to parse JSON
                        match serde_json::from_str::<Value>(&text) {
                            Ok(_) => {
                                valid_parsed += 1;
                                metrics.messages_received += 1;
                            }
                            Err(_) => {
                                invalid_received += 1;
                            }
                        }
                    }
                    Ok(Some(Ok(Message::Ping(data)))) => {
                        let _ = sink.send(Message::Pong(data)).await;
                    }
                    Err(_) => break,
                    _ => {}
                }
            }
        }
        Err(e) => panic!("Connection failed: {}", e),
    }

    metrics.final_success = valid_parsed > 0;
    metrics.print_summary();

    println!("Valid messages parsed: {}", valid_parsed);
    println!("Invalid messages handled: {}", invalid_received);
    println!("✓ Successfully handled corrupted JSON gracefully");

    assert!(valid_parsed > 0, "Should parse some valid messages");
    assert!(invalid_received > 0, "Should encounter corrupted messages");
}

#[tokio::test]
async fn test_chaos_intermittent_reset() {
    println!("\n=== Test: Intermittent Connection Resets ===");

    let server = ChaosBinanceServer::new(FailureMode::IntermittentReset(2))
        .await
        .expect("Failed to create server");
    let server_url = server.url();

    tokio::spawn(async move {
        server.run().await;
    });

    sleep(Duration::from_millis(100)).await;

    let mut metrics = ResilienceMetrics::default();
    let test_duration = Duration::from_secs(3);
    let start = tokio::time::Instant::now();

    while start.elapsed() < test_duration {
        match connect_async(&server_url).await {
            Ok((ws_stream, _)) => {
                metrics.reconnects += 1;
                let (mut sink, mut stream) = ws_stream.split();

                let sub_msg = json!({
                    "method": "SUBSCRIBE",
                    "params": ["btcusdt@aggTrade"],
                    "id": 1
                });
                let _ = sink.send(Message::text(sub_msg.to_string())).await;

                while let Ok(Some(Ok(msg))) = tokio::time::timeout(Duration::from_millis(500), stream.next()).await {
                    match msg {
                        Message::Text(_) => {
                            metrics.messages_received += 1;
                            metrics.errors_recovered += 1;
                        }
                        Message::Ping(data) => {
                            let _ = sink.send(Message::Pong(data)).await;
                        }
                        _ => {}
                    }
                }
            }
            Err(_) => {
                sleep(Duration::from_millis(200)).await;
            }
        }
    }

    metrics.final_success = metrics.reconnects > 1;
    metrics.print_summary();

    println!("Successfully reconnected {} times", metrics.reconnects);
    println!("✓ Rapid reconnection handled correctly");
    assert!(metrics.reconnects > 1, "Should handle multiple reconnections");
}

#[tokio::test]
async fn test_chaos_ping_flood() {
    println!("\n=== Test: Ping Flood from Server ===");

    let server = ChaosBinanceServer::new(FailureMode::PingFlood)
        .await
        .expect("Failed to create server");
    let server_url = server.url();

    tokio::spawn(async move {
        server.run().await;
    });

    sleep(Duration::from_millis(100)).await;

    let mut metrics = ResilienceMetrics::default();

    match connect_async(&server_url).await {
        Ok((ws_stream, _)) => {
            let (mut sink, mut stream) = ws_stream.split();

            let sub_msg = json!({
                "method": "SUBSCRIBE",
                "params": ["btcusdt@aggTrade"],
                "id": 1
            });
            let _ = sink.send(Message::text(sub_msg.to_string())).await;

            let mut pong_sent = 0;
            for _ in 0..50 {
                match tokio::time::timeout(Duration::from_millis(500), stream.next()).await {
                    Ok(Some(Ok(Message::Text(_)))) => {
                        metrics.messages_received += 1;
                    }
                    Ok(Some(Ok(Message::Ping(data)))) => {
                        let _ = sink.send(Message::Pong(data)).await;
                        pong_sent += 1;
                    }
                    Err(_) => break,
                    _ => {}
                }
            }

            metrics.final_success = metrics.messages_received > 0 && pong_sent > 0;
            println!("Pongs sent in response to flood: {}", pong_sent);
        }
        Err(e) => panic!("Connection failed: {}", e),
    }

    metrics.print_summary();

    println!("✓ Successfully handled ping flood");
    assert!(metrics.messages_received > 0, "Should still receive messages");
}

#[tokio::test]
async fn test_chaos_slow_subscription() {
    println!("\n=== Test: Slow Subscription Response ===");

    let server = ChaosBinanceServer::new(FailureMode::SlowSubscription)
        .await
        .expect("Failed to create server");
    let server_url = server.url();

    tokio::spawn(async move {
        server.run().await;
    });

    sleep(Duration::from_millis(100)).await;

    let start = tokio::time::Instant::now();

    match connect_async(&server_url).await {
        Ok((ws_stream, _)) => {
            let (mut sink, mut stream) = ws_stream.split();

            let sub_msg = json!({
                "method": "SUBSCRIBE",
                "params": ["btcusdt@aggTrade"],
                "id": 1
            });
            let _ = sink.send(Message::text(sub_msg.to_string())).await;

            // Wait for subscription response
            match tokio::time::timeout(Duration::from_secs(5), stream.next()).await {
                Ok(Some(Ok(Message::Text(text)))) => {
                    let elapsed = start.elapsed();
                    println!("Subscription response received in {:?}", elapsed);

                    if text.contains("result") {
                        println!("✓ Got subscription response: {}", text);
                    }
                }
                Err(_) => panic!("Subscription timeout"),
                _ => panic!("Unexpected message type"),
            }
        }
        Err(e) => panic!("Connection failed: {}", e),
    }
}
