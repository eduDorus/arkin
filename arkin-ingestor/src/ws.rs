use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{interval, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

/// Configuration for WebSocket connection
#[derive(Clone)]
pub struct WsConfig {
    pub url: String,
    pub streams: Vec<(String, String)>, // (message, stream_id)
    pub reconnect_backoff_ms: u64,
    pub max_reconnect_attempts: u32,
    pub ping_interval_secs: u64, // Interval to send pings to server
    pub pong_timeout_secs: u64,
    pub stale_connection_timeout_secs: u64, // Detect stale connections (no messages received)
    pub send_ping_interval_secs: u64,       // How often to send pings (e.g., 20s for Coinbase)
}

impl WsConfig {
    pub fn binance_usds_futures(streams: Vec<(String, String)>) -> Self {
        Self {
            url: "wss://fstream.binance.com/ws".to_string(),
            streams,
            reconnect_backoff_ms: 1000,
            max_reconnect_attempts: 10,
            ping_interval_secs: 180,           // 3 minutes (Binance sends ping every 3 min)
            pong_timeout_secs: 600,            // 10 minutes (Binance disconnects after 10 min no pong)
            stale_connection_timeout_secs: 30, // Reconnect if no messages for 30 seconds
            send_ping_interval_secs: 0,        // Don't send pings (Binance sends them to us)
        }
    }
}

/// Robust WebSocket client for Binance market data streams
pub struct WsClient {
    config: WsConfig,
    broadcaster: mpsc::UnboundedSender<Value>,
    reconnect_attempts: u32,
    is_connected: Arc<AtomicBool>,
    last_message_timestamp: Arc<AtomicU64>, // Track last message time for stale detection
    stream_ids: Vec<String>,                // Store stream IDs for logging
}

impl WsClient {
    pub fn new(config: WsConfig) -> (Self, mpsc::UnboundedReceiver<Value>) {
        let (tx, rx) = mpsc::unbounded_channel();
        let stream_ids = config.streams.iter().map(|(_, id)| id.clone()).collect();
        let client = Self {
            config,
            broadcaster: tx,
            reconnect_attempts: 0,
            is_connected: Arc::new(AtomicBool::new(false)),
            last_message_timestamp: Arc::new(AtomicU64::new(0)),
            stream_ids,
        };
        (client, rx)
    }

    /// Start the WebSocket connection with automatic reconnection
    pub async fn run(&mut self) -> Result<(), String> {
        loop {
            match self.connect_and_handle().await {
                Ok(_) => {
                    self.reconnect_attempts = 0;
                    info!("WebSocket connection closed normally");
                    sleep(Duration::from_millis(self.config.reconnect_backoff_ms)).await;
                }
                Err(e) => {
                    self.reconnect_attempts += 1;
                    self.is_connected.store(false, Ordering::SeqCst);

                    if self.reconnect_attempts >= self.config.max_reconnect_attempts {
                        error!(
                            "Max reconnection attempts ({}) exceeded: {}",
                            self.config.max_reconnect_attempts, e
                        );
                        return Err(e);
                    }

                    // Calculate backoff: first attempt no wait (0ms), then 1s, then exponential
                    let backoff = if self.reconnect_attempts == 1 {
                        Duration::from_millis(0)
                    } else {
                        Duration::from_millis(
                            self.config.reconnect_backoff_ms * (2u64.pow(self.reconnect_attempts - 2)),
                        )
                    };
                    warn!(
                        "Connection failed (attempt {}/{}), retrying in {:?}: {}",
                        self.reconnect_attempts, self.config.max_reconnect_attempts, backoff, e
                    );
                    sleep(backoff).await;
                }
            }
        }
    }

    async fn connect_and_handle(&mut self) -> Result<(), String> {
        info!("Connecting to {}", self.config.url);
        let (ws_stream, _) = connect_async(self.config.url.as_str())
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        let (mut sink, mut stream) = ws_stream.split();

        info!("Connected to WebSocket");
        self.is_connected.store(true, Ordering::SeqCst);
        self.last_message_timestamp.store(0, Ordering::SeqCst); // Reset on new connection

        // Subscribe to all streams
        self.subscribe_to_streams(&mut sink).await?;

        // Create a channel for sending pings
        let (ping_tx, mut ping_rx) = tokio::sync::mpsc::unbounded_channel::<()>();

        // Spawn ping/pong task
        let config_clone = self.config.clone();
        let is_connected_clone = self.is_connected.clone();
        let stream_ids_clone = self.stream_ids.clone();
        let _ping_task = tokio::spawn(async move {
            Self::ping_pong_loop(config_clone, is_connected_clone, stream_ids_clone, ping_tx).await
        });

        // Spawn stale connection detector
        let config_clone = self.config.clone();
        let is_connected_clone = self.is_connected.clone();
        let last_message_ts_clone = self.last_message_timestamp.clone();
        let stream_ids_clone = self.stream_ids.clone();
        let _stale_detector = tokio::spawn(async move {
            Self::stale_connection_detector(config_clone, is_connected_clone, last_message_ts_clone, stream_ids_clone)
                .await
        });

        // Main message handling loop with stale connection timeout
        let stale_timeout = Duration::from_secs(self.config.stale_connection_timeout_secs);

        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                result = tokio::time::timeout(stale_timeout, stream.next()) => {
                    match result {
                        Ok(Some(Ok(msg))) => {
                            // Update last message timestamp to current time
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            self.last_message_timestamp.store(now, Ordering::SeqCst);

                            match msg {
                                Message::Text(text) => {
                                    // Parse and broadcast the JSON message
                                    match serde_json::from_str::<Value>(&text) {
                                        Ok(json_msg) => {
                                            let _ = self.broadcaster.send(json_msg);
                                        }
                                        Err(e) => {
                                            warn!("Failed to parse message: {}", e);
                                        }
                                    }
                                }
                                Message::Binary(_data) => {
                                    warn!("Received unexpected binary message");
                                }
                                Message::Ping(data) => {
                                    info!(
                                        "Received ping from server [{}], responding with pong",
                                        self.stream_ids.join(", ")
                                    );
                                    sink.send(Message::Pong(data))
                                        .await
                                        .map_err(|e| format!("Failed to send pong: {}", e))?;
                                }
                                Message::Pong(_data) => {
                                    info!("Received pong from server [{}]", self.stream_ids.join(", "));
                                }
                                Message::Close(frame) => {
                                    info!("Received close frame: {:?}", frame);
                                    return Ok(());
                                }
                                Message::Frame(_) => {
                                    // Frame variant shouldn't occur in normal operation
                                }
                            }
                        }
                        Ok(Some(Err(e))) => {
                            error!("WebSocket error: {}", e);
                            return Err(format!("WebSocket error: {}", e));
                        }
                        Ok(None) => {
                            info!("WebSocket stream ended");
                            return Ok(());
                        }
                        Err(_) => {
                            // Timeout: no messages received for stale_connection_timeout_secs
                            error!(
                                "Stale connection detected - no messages received within {:?}. Reconnecting...",
                                stale_timeout
                            );
                            return Err("Stale connection - no messages received".to_string());
                        }
                    }
                }

                // Handle ping requests from ping_pong_loop
                _ = ping_rx.recv() => {
                    use tokio_tungstenite::tungstenite::Bytes;
                    if sink.send(Message::Ping(Bytes::new())).await.is_err() {
                        warn!("Failed to send ping to [{}]", self.stream_ids.join(", "));
                        return Err("Failed to send ping".to_string());
                    }
                }
            }
        }
    }

    async fn subscribe_to_streams(
        &self,
        sink: &mut futures::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
            Message,
        >,
    ) -> Result<(), String> {
        // Each stream in config.streams is already a complete JSON subscription message
        for (msg_text, stream_id) in &self.config.streams {
            info!("Subscribing to [{}]: {}", stream_id, msg_text);

            // Send each subscription message directly
            sink.send(Message::text(msg_text.clone()))
                .await
                .map_err(|e| format!("Failed to send subscription for [{}]: {}", stream_id, e))?;

            info!("Subscription sent for [{}]", stream_id);
        }
        Ok(())
    }

    /// Ping/pong loop to send periodic pings to keep connection alive
    async fn ping_pong_loop(
        config: WsConfig,
        is_connected: Arc<AtomicBool>,
        stream_ids: Vec<String>,
        ping_tx: tokio::sync::mpsc::UnboundedSender<()>,
    ) {
        // If send_ping_interval_secs is 0, don't send pings - just keep the task alive to prevent channel closure
        if config.send_ping_interval_secs == 0 {
            // Keep the task alive indefinitely
            loop {
                if !is_connected.load(Ordering::SeqCst) {
                    break;
                }
                sleep(Duration::from_secs(60)).await;
            }
            return;
        }

        let mut ping_interval = interval(Duration::from_secs(config.send_ping_interval_secs));
        let stream_id_str = stream_ids.join(", ");

        loop {
            ping_interval.tick().await;
            if !is_connected.load(Ordering::SeqCst) {
                break;
            }
            info!("Sending ping to [{}] to keep connection alive", stream_id_str);
            let _ = ping_tx.send(());
        }
    }

    /// Stale connection detector - monitors if stream is receiving messages
    /// If no messages are received for stale_connection_timeout_secs, the connection
    /// is considered stale and will be reconnected
    async fn stale_connection_detector(
        config: WsConfig,
        is_connected: Arc<AtomicBool>,
        last_message_timestamp: Arc<AtomicU64>,
        stream_ids: Vec<String>,
    ) {
        let check_interval = Duration::from_secs(5); // Check every 5 seconds
        let mut checker = interval(check_interval);
        let stream_id_str = stream_ids.join(", ");

        loop {
            checker.tick().await;

            if !is_connected.load(Ordering::SeqCst) {
                continue;
            }

            let last_ts = last_message_timestamp.load(Ordering::SeqCst);

            // If we haven't received any messages yet, skip
            if last_ts == 0 {
                continue;
            }

            // Get current time and check elapsed duration
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            let elapsed = now.saturating_sub(last_ts);

            if elapsed as u64 > config.stale_connection_timeout_secs {
                warn!(
                    "Stream appears stale [{}] - no messages received for {}s (stale timeout: {}s)",
                    stream_id_str, elapsed, config.stale_connection_timeout_secs
                );
                // Signal that we need to reconnect by marking as disconnected
                is_connected.store(false, Ordering::SeqCst);
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }
}
