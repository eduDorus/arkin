use futures::{SinkExt, StreamExt};
use serde_json::Value;
use std::cmp;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{interval, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

/// Error types categorized for statistics tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Connection failed to establish
    ConnectionFailed,
    /// WebSocket protocol error
    WebSocketError,
    /// Failed to parse JSON message
    ParseError,
    /// Failed to send pong response
    PongSendFailed,
    /// Failed to send ping
    PingSendFailed,
    /// Failed to send subscription
    SubscriptionFailed,
    /// Stale connection detected
    StaleConnection,
    /// Unexpected binary message received
    UnexpectedBinary,
    /// Generic/Other error
    Other,
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConnectionFailed => write!(f, "connection_failed"),
            Self::WebSocketError => write!(f, "websocket_error"),
            Self::ParseError => write!(f, "parse_error"),
            Self::PongSendFailed => write!(f, "pong_send_failed"),
            Self::PingSendFailed => write!(f, "ping_send_failed"),
            Self::SubscriptionFailed => write!(f, "subscription_failed"),
            Self::StaleConnection => write!(f, "stale_connection"),
            Self::UnexpectedBinary => write!(f, "unexpected_binary"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Error statistics for WebSocket connection
#[derive(Debug, Clone)]
pub struct ErrorStats {
    /// Total number of errors
    pub total_errors: u64,
    /// Error counts by category
    pub errors_by_type: Vec<(String, u64)>,
    /// Last error message details (type, message)
    pub last_error: Option<(String, String)>,
}

impl std::fmt::Display for ErrorStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Total Errors: {} | Breakdown: {}",
            self.total_errors,
            self.errors_by_type
                .iter()
                .map(|(category, count)| format!("{}: {}", category, count))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

/// Tracks error statistics for the WebSocket client
#[allow(dead_code)]
struct ErrorTracker {
    total_errors: Arc<AtomicU64>,
    connection_failed: Arc<AtomicU64>,
    websocket_error: Arc<AtomicU64>,
    parse_error: Arc<AtomicU64>,
    pong_send_failed: Arc<AtomicU64>,
    ping_send_failed: Arc<AtomicU64>,
    subscription_failed: Arc<AtomicU64>,
    stale_connection: Arc<AtomicU64>,
    unexpected_binary: Arc<AtomicU64>,
    other_error: Arc<AtomicU64>,
    last_error_category: Arc<tokio::sync::Mutex<Option<String>>>,
    last_error_message: Arc<tokio::sync::Mutex<Option<String>>>,
}

impl ErrorTracker {
    fn new() -> Self {
        Self {
            total_errors: Arc::new(AtomicU64::new(0)),
            connection_failed: Arc::new(AtomicU64::new(0)),
            websocket_error: Arc::new(AtomicU64::new(0)),
            parse_error: Arc::new(AtomicU64::new(0)),
            pong_send_failed: Arc::new(AtomicU64::new(0)),
            ping_send_failed: Arc::new(AtomicU64::new(0)),
            subscription_failed: Arc::new(AtomicU64::new(0)),
            stale_connection: Arc::new(AtomicU64::new(0)),
            unexpected_binary: Arc::new(AtomicU64::new(0)),
            other_error: Arc::new(AtomicU64::new(0)),
            last_error_category: Arc::new(tokio::sync::Mutex::new(None)),
            last_error_message: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    fn record_error(&self, category: ErrorCategory) {
        self.total_errors.fetch_add(1, Ordering::SeqCst);
        match category {
            ErrorCategory::ConnectionFailed => self.connection_failed.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::WebSocketError => self.websocket_error.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::ParseError => self.parse_error.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::PongSendFailed => self.pong_send_failed.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::PingSendFailed => self.ping_send_failed.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::SubscriptionFailed => self.subscription_failed.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::StaleConnection => self.stale_connection.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::UnexpectedBinary => self.unexpected_binary.fetch_add(1, Ordering::SeqCst),
            ErrorCategory::Other => self.other_error.fetch_add(1, Ordering::SeqCst),
        };
    }

    fn record_error_with_message(&self, category: ErrorCategory, message: String) {
        self.record_error(category);

        let category_str = category.to_string();
        let last_category = self.last_error_category.clone();
        let last_message = self.last_error_message.clone();

        // Spawn a task to update the error details (non-blocking)
        tokio::spawn(async move {
            let mut cat = last_category.lock().await;
            *cat = Some(category_str);

            let mut msg = last_message.lock().await;
            *msg = Some(message);
        });
    }

    fn get_stats(&self) -> ErrorStats {
        let mut errors_by_type = vec![
            ("connection_failed".to_string(), self.connection_failed.load(Ordering::SeqCst)),
            ("websocket_error".to_string(), self.websocket_error.load(Ordering::SeqCst)),
            ("parse_error".to_string(), self.parse_error.load(Ordering::SeqCst)),
            ("pong_send_failed".to_string(), self.pong_send_failed.load(Ordering::SeqCst)),
            ("ping_send_failed".to_string(), self.ping_send_failed.load(Ordering::SeqCst)),
            (
                "subscription_failed".to_string(),
                self.subscription_failed.load(Ordering::SeqCst),
            ),
            ("stale_connection".to_string(), self.stale_connection.load(Ordering::SeqCst)),
            ("unexpected_binary".to_string(), self.unexpected_binary.load(Ordering::SeqCst)),
            ("other".to_string(), self.other_error.load(Ordering::SeqCst)),
        ];

        // Filter out zero counts for cleaner output
        errors_by_type.retain(|(_, count)| *count > 0);

        // Get last error details (non-blocking read attempt)
        let last_error = if let Ok(cat) = self.last_error_category.try_lock() {
            if let Ok(msg) = self.last_error_message.try_lock() {
                match (cat.clone(), msg.clone()) {
                    (Some(c), Some(m)) => Some((c, m)),
                    _ => None,
                }
            } else {
                None
            }
        } else {
            None
        };

        ErrorStats {
            total_errors: self.total_errors.load(Ordering::SeqCst),
            errors_by_type,
            last_error,
        }
    }
}

/// Configuration for WebSocket connection
#[derive(Clone)]
pub struct WsConfig {
    pub url: String,
    pub streams: Vec<(String, String)>, // (message, stream_id)
    pub reconnect_backoff_ms: u64,
    pub max_reconnect_backoff_ms: u64,
    pub ping_interval_secs: u64,            // Interval to send pings to server
    pub stale_connection_timeout_secs: u64, // Detect stale connections (no messages received)
}

/// Robust WebSocket client for Binance market data streams
pub struct WsClient {
    config: WsConfig,
    broadcaster: mpsc::UnboundedSender<Value>,
    reconnect_attempts: u32,
    is_connected: Arc<AtomicBool>,
    last_message_timestamp: Arc<AtomicU64>, // Track last message time for stale detection
    stream_ids: Vec<String>,                // Store stream IDs for logging
    error_tracker: ErrorTracker,            // Track error statistics
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
            error_tracker: ErrorTracker::new(),
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
                    self.error_tracker
                        .record_error_with_message(ErrorCategory::ConnectionFailed, e.clone());
                    self.is_connected.store(false, Ordering::SeqCst);

                    // Calculate backoff: first attempt no wait (0ms), then 1s, then exponential
                    let backoff = if self.reconnect_attempts == 1 {
                        Duration::from_millis(0)
                    } else {
                        let backoff = cmp::min(
                            self.config.reconnect_backoff_ms * (2u64.pow(self.reconnect_attempts - 2)),
                            self.config.max_reconnect_backoff_ms,
                        );
                        Duration::from_millis(backoff)
                    };
                    warn!(
                        "Connection failed (attempt {}), retrying in {:?}: {}",
                        self.reconnect_attempts, backoff, e
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
                                            self.error_tracker.record_error_with_message(
                                                ErrorCategory::ParseError,
                                                e.to_string(),
                                            );
                                        }
                                    }
                                }
                                Message::Binary(_data) => {
                                    warn!("Received unexpected binary message");
                                    self.error_tracker.record_error(ErrorCategory::UnexpectedBinary);
                                }
                                Message::Ping(data) => {
                                    info!(
                                        "Received ping from server [{}], responding with pong",
                                        self.stream_ids.join(", ")
                                    );
                                    if sink.send(Message::Pong(data))
                                        .await
                                        .is_err() {
                                        error!("Failed to send pong");
                                        self.error_tracker.record_error_with_message(
                                            ErrorCategory::PongSendFailed,
                                            "Failed to send pong response".to_string(),
                                        );
                                        return Err("Failed to send pong".to_string());
                                    }
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
                            self.error_tracker.record_error_with_message(
                                ErrorCategory::WebSocketError,
                                e.to_string(),
                            );
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
                            self.error_tracker.record_error_with_message(
                                ErrorCategory::StaleConnection,
                                format!("No messages for {:?}", stale_timeout),
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
                        self.error_tracker.record_error_with_message(
                            ErrorCategory::PingSendFailed,
                            "Failed to send ping".to_string(),
                        );
                        return Err("Failed to send ping".to_string());
                    }
                }
            }
        }
    }

    async fn subscribe_to_streams(
        &mut self,
        sink: &mut futures::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
            Message,
        >,
    ) -> Result<(), String> {
        // Each stream in config.streams is already a complete JSON subscription message
        for (msg_text, stream_id) in &self.config.streams {
            info!("Subscribing to [{}]: {}", stream_id, msg_text);

            // Send each subscription message directly
            match sink.send(Message::text(msg_text.clone())).await {
                Ok(_) => {
                    info!("Subscription sent for [{}]", stream_id);
                }
                Err(e) => {
                    error!("Failed to send subscription for [{}]: {}", stream_id, e);
                    self.error_tracker.record_error_with_message(
                        ErrorCategory::SubscriptionFailed,
                        format!("Failed to subscribe to [{}]: {}", stream_id, e),
                    );
                    return Err(format!("Failed to send subscription for [{}]: {}", stream_id, e));
                }
            }
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
        // If ping_interval_secs is 0, don't send pings - just keep the task alive to prevent channel closure
        if config.ping_interval_secs == 0 {
            // Keep the task alive indefinitely
            loop {
                if !is_connected.load(Ordering::SeqCst) {
                    break;
                }
                sleep(Duration::from_secs(60)).await;
            }
            return;
        }

        let mut ping_interval = interval(Duration::from_secs(config.ping_interval_secs));
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

    /// Get the current error statistics
    pub fn error_stats(&self) -> ErrorStats {
        self.error_tracker.get_stats()
    }
}
