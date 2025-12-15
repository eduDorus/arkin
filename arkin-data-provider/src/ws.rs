use anyhow::Result;
use arkin_core::Event;
use futures::{SinkExt, StreamExt};
use kanal::AsyncSender;
use std::cmp;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use time::UtcDateTime;
use tokio::sync::mpsc;
use tokio::time::{interval, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_util::bytes::Bytes;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use typed_builder::TypedBuilder;

use crate::WebSocketProvider;

#[derive(TypedBuilder)]
pub struct WsClient {
    provider: Box<dyn WebSocketProvider>,
    #[builder(default = 0)]
    reconnect_attempts: u32,
    #[builder(default = Arc::new(AtomicBool::new(false)))]
    is_connected: Arc<AtomicBool>,
    #[builder(default = Arc::new(AtomicU64::new(0)))]
    last_message_timestamp: Arc<AtomicU64>, // Track last message time for stale detection
    #[builder(default = 30)]
    stale_connection_timeout_secs: u64,
    #[builder(default = 1000)]
    reconnect_backoff_ms: u64,
    #[builder(default = 30000)]
    max_reconnect_backoff_ms: u64,
    #[builder(default = 15)]
    ping_interval_secs: u64,
}

impl WsClient {
    pub fn new(provider: Box<dyn WebSocketProvider>) -> Self {
        Self {
            provider,
            reconnect_attempts: 0,
            is_connected: Arc::new(AtomicBool::new(false)),
            last_message_timestamp: Arc::new(AtomicU64::new(0)),
            stale_connection_timeout_secs: 5, // Default stale timeout
            reconnect_backoff_ms: 1000,       // backoff 1s
            max_reconnect_backoff_ms: 30000,  // Max backoff 30
            ping_interval_secs: 15,           // Default ping interval 15s
        }
    }

    pub fn is_connected(&self) -> bool {
        self.is_connected.load(Ordering::SeqCst)
    }

    /// Start the WebSocket connection with automatic reconnection
    pub async fn run(&mut self, sender: AsyncSender<Event>, shutdown: CancellationToken) {
        // Setup the provider
        if let Err(e) = self.provider.setup().await {
            error!("Failed to setup provider: {:?}", e);
            return;
        }

        loop {
            match self.connect_and_handle(sender.clone(), shutdown.clone()).await {
                Ok(_) => {
                    info!("WebSocket connection closed normally");
                    break;
                }
                Err(e) => {
                    self.reconnect_attempts += 1;
                    self.is_connected.store(false, Ordering::SeqCst);

                    // Calculate backoff: first attempt no wait (0ms), then 1s, then exponential
                    let backoff = if self.reconnect_attempts == 1 {
                        Duration::from_millis(0)
                    } else {
                        let backoff = cmp::min(
                            self.reconnect_backoff_ms * (2u64.pow(self.reconnect_attempts - 2)),
                            self.max_reconnect_backoff_ms,
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

        // Teardown the provider
        if let Err(e) = self.provider.teardown().await {
            error!("Failed to teardown provider: {:?}", e);
        }
    }

    async fn connect_and_handle(
        &mut self,
        sender: AsyncSender<Event>,
        shutdown: CancellationToken,
    ) -> Result<(), String> {
        info!("Connecting to {}", self.provider.url());
        let (ws_stream, _) = connect_async(self.provider.url())
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;
        let (mut sink, mut stream) = ws_stream.split();

        info!("Connected to WebSocket");
        self.is_connected.store(true, Ordering::SeqCst);
        self.last_message_timestamp.store(0, Ordering::SeqCst); // Reset on new connection
        self.reconnect_attempts = 0;

        // Subscribe to all streams
        self.subscribe_to_streams(&mut sink).await?;

        // Create a channel for sending pings
        let (ping_tx, mut ping_rx) = mpsc::unbounded_channel::<()>();

        // Spawn ping/pong task
        let is_connected = self.is_connected.clone();
        let ping_interval = self.ping_interval_secs;
        let _ping_task = tokio::spawn(async move { Self::ping_pong_loop(is_connected, ping_interval, ping_tx).await });

        // Spawn stale connection detector
        let is_connected_clone = self.is_connected.clone();
        let last_message_timestamp = self.last_message_timestamp.clone();
        let stale_connection_timeout_secs = self.stale_connection_timeout_secs;
        let _stale_detector = tokio::spawn(async move {
            Self::stale_connection_detector(is_connected_clone, last_message_timestamp, stale_connection_timeout_secs)
                .await
        });

        // Main message handling loop with stale connection timeout
        let stale_timeout = Duration::from_secs(self.stale_connection_timeout_secs);

        loop {
            tokio::select! {
                // Handle incoming WebSocket messages
                result = tokio::time::timeout(stale_timeout, stream.next()) => {
                    match result {
                        Ok(Some(Ok(msg))) => {
                            // Update last message timestamp to current time
                            let now = UtcDateTime::now().unix_timestamp() as u64;
                            self.last_message_timestamp.store(now, Ordering::SeqCst);

                            match msg {
                                Message::Text(text) => {
                                    // Parse and broadcast the JSON message
                                    match self.provider.parse(&text).await {
                                        Ok(Some(event)) => {
                                            debug!("Parsed event: {}", event);
                                            if sender.send(event).await.is_err() {
                                                warn!("Failed to send event");
                                            }
                                        }
                                        Ok(None) => {
                                            // Message handled but no event produced (e.g. heartbeat, ignored message)
                                            debug!("Message handled but ignored: {}", text);
                                        }
                                        Err(e) => {
                                            warn!("Failed to parse message: {} - Error: {:?}", text, e);
                                        }
                                    }
                                }
                                Message::Binary(_data) => {
                                    warn!("Received unexpected binary message");
                                }
                                Message::Ping(data) => {
                                    debug!("Received ping from server, responding with pong");
                                    if sink.send(Message::Pong(data))
                                        .await
                                        .is_err() {
                                        error!("Failed to send pong");
                                        return Err("Failed to send pong".to_string());
                                    }
                                }
                                Message::Pong(_data) => {
                                    info!("Received pong from server {}", self.provider.name());
                                }
                                Message::Close(frame) => {
                                    info!("Received close frame: {:?}", frame);
                                    return Err("WebSocket closed by server".to_string());
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
                            return Err("WebSocket closed by server".to_string());
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
                    if let Err(e) = sink.send(Message::Ping(Bytes::new())).await {
                        error!("Failed to send ping: {}", e);
                        return Err("Failed to send ping".to_string());
                    } else {
                        info!("Sent ping to server {}", self.provider.name());
                    }
                }

                // Handle shutdown signal
                _ = shutdown.cancelled() => {
                    info!("Shutdown signal received, closing WebSocket connection");
                    if let Err(e) = sink.send(Message::Close(None)).await {
                        error!("Failed to send close message: {}", e);
                    }
                    self.is_connected.store(false, Ordering::SeqCst);
                    return Ok(());
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
        if let Some(subscription_msg) = self.provider.subscribe_msg() {
            match sink.send(Message::text(subscription_msg)).await {
                Ok(_) => {
                    info!("Subscription sent for [{}]", self.provider.name());
                }
                Err(e) => {
                    error!("Failed to send subscription for [{}]: {}", self.provider.name(), e);
                }
            }
        }
        Ok(())
    }

    async fn ping_pong_loop(
        is_connected: Arc<AtomicBool>,
        ping_interval_secs: u64,
        ping_tx: mpsc::UnboundedSender<()>,
    ) {
        // If ping_interval_secs is 0, don't send pings - just keep the task alive to prevent channel closure
        if ping_interval_secs == 0 {
            return;
        }

        let mut ping_interval = interval(Duration::from_secs(ping_interval_secs));
        loop {
            ping_interval.tick().await;
            if !is_connected.load(Ordering::SeqCst) {
                break;
            }
            let _ = ping_tx.send(());
        }
    }

    async fn stale_connection_detector(
        is_connected: Arc<AtomicBool>,
        last_message_timestamp: Arc<AtomicU64>,
        stale_connection_timeout_sec: u64,
    ) {
        let check_interval = Duration::from_secs(1); // Check every 5 seconds
        let mut checker = interval(check_interval);

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
            let now = UtcDateTime::now().unix_timestamp() as u64;

            let elapsed = now.saturating_sub(last_ts);

            if elapsed as u64 > stale_connection_timeout_sec {
                warn!(
                    "Stream appears stale - no messages received for {}s (stale timeout: {}s)",
                    elapsed, stale_connection_timeout_sec
                );
                // Signal that we need to reconnect by marking as disconnected
                is_connected.store(false, Ordering::SeqCst);
            } else {
                debug!("Stream healthy - last message received {}s ago", elapsed);
            }
        }
    }
}
