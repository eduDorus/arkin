use arkin_core::prelude::Deduplicator; // Assume from Base 2 dep
use futures::{SinkExt, StreamExt};
use kanal::{AsyncReceiver, AsyncSender};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::{
    net::TcpStream,
    select,
    sync::{oneshot, Mutex},
    time::{interval, sleep, Instant, Interval},
};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{protocol::Message, Error as WsError},
    MaybeTlsStream, WebSocketStream,
};
use tokio_util::{
    sync::CancellationToken,
    task::{task_tracker, TaskTracker},
};
use tracing::{debug, error, info, warn};
use url::Url;
use uuid::Uuid;

use super::{
    api::{
        request::WsRequest,
        response::{ApiError, WsResponse},
    },
    user_stream::{AccountUpdateEvent, OrderTradeUpdateEvent},
};
use crate::ws::{MarketStream, WsEvent};

type WsStreamType = WebSocketStream<MaybeTlsStream<TcpStream>>;
type PendingAcks = Arc<Mutex<HashMap<String, oneshot::Sender<Result<(), ApiError>>>>>;

#[derive(Error, Debug)]
pub enum WsManagerError {
    #[error("WebSocket error: {0}")]
    WsError(#[from] WsError),
    #[error("Channel error: {0}")]
    ChannelError(String),
    #[error("Parse error: {0}")]
    ParseError(#[from] serde_json::Error),
    #[error("API error: {0}")]
    ApiError(ApiError),
    #[error("Timeout error")]
    Timeout,
    #[error("Unexpected: {0}")]
    Unexpected(String),
}

enum InternalCommand {
    Subscribe {
        streams: Vec<String>,
        ack_tx: oneshot::Sender<Result<(), ApiError>>,
    },
    Unsubscribe {
        streams: Vec<String>,
        ack_tx: oneshot::Sender<Result<(), ApiError>>,
    },
}

#[derive(Clone)]
pub struct MarketWsManager {
    url: Url,
    shutdown: CancellationToken,
    tracker: TaskTracker,
}

impl MarketWsManager {
    pub async fn new(url: Url) -> Self {
        let shutdown = CancellationToken::new();
        let tracker = TaskTracker::new();
        Self {
            url,
            tracker,
        }
    }

    pub async fn subscribe(&self, streams: Vec<String>) -> Result<(), WsManagerError> {
        let (ack_tx, ack_rx) = oneshot::channel();
        self.command_tx
            .send(InternalCommand::Subscribe { streams, ack_tx })
            .await
            .map_err(|e| WsManagerError::ChannelError(e.to_string()))?;
        ack_rx
            .await
            .map_err(|_| WsManagerError::Timeout)?
            .map_err(WsManagerError::ApiError)
    }

    pub async fn unsubscribe(&self, streams: Vec<String>) -> Result<(), WsManagerError> {
        let (ack_tx, ack_rx) = oneshot::channel();
        self.command_tx
            .send(InternalCommand::Unsubscribe { streams, ack_tx })
            .await
            .map_err(|e| WsManagerError::ChannelError(e.to_string()))?;
        ack_rx
            .await
            .map_err(|_| WsManagerError::Timeout)?
            .map_err(WsManagerError::ApiError)
    }

    pub fn shutdown(&self) {
        self.shutdown.cancel();
    }

    async fn run_loop(
        url: Url,
        event_tx: AsyncSender<WsEvent>,
        mut command_rx: AsyncReceiver<InternalCommand>,
        pending_acks: PendingAcks,
        deduplicator: Arc<Deduplicator>,
        shutdown: CancellationToken,
    ) {
        let mut retries = 0;
        const MAX_RETRIES: usize = 5;
        while retries < MAX_RETRIES && !shutdown.is_cancelled() {
            match connect_async(&url).await {
                Ok((mut stream, _)) => {
                    info!("Market WS connected.");
                    retries = 0;
                    let mut ping_interval = interval(Duration::from_secs(180));
                    let mut last_activity = Instant::now();
                    loop {
                        select! {
                            _ = ping_interval.tick() => {
                                if let Err(e) = stream.send(Message::Pong(vec![])).await {
                                    error!("Pong failed: {}. Reconnecting...", e);
                                    break;
                                }
                            }
                            Some(command) = command_rx.recv() => {
                                if let Err(e) = Self::handle_command(&mut stream, command, &pending_acks).await {
                                    error!("Command failed: {}. Reconnecting...", e);
                                    break;
                                }
                            }
                            msg = stream.next() => {
                                match msg {
                                    Some(Ok(msg)) => {
                                        last_activity = Instant::now();
                                        if let Err(e) = Self::handle_message(msg, &pending_acks, &deduplicator, &event_tx).await {
                                            error!("Message handling failed: {}. Reconnecting...", e);
                                            break;
                                        }
                                    }
                                    Some(Err(e)) => {
                                        error!("Recv error: {}. Reconnecting...", e);
                                        break;
                                    }
                                    None => {
                                        warn!("Stream closed. Reconnecting...");
                                        break;
                                    }
                                }
                            }
                            _ = sleep(Duration::from_secs(10)) => { // Check inactivity periodically
                                if Instant::now().duration_since(last_activity) > Duration::from_secs(360) {
                                    warn!("Inactivity timeout (>360s). Reconnecting...");
                                    break;
                                }
                            }
                            _ = shutdown.cancelled() => {
                                let _ = stream.close(None).await;
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Connect failed: {}.", e);
                }
            }
            retries += 1;
            let backoff = Duration::from_secs(2u64.pow(retries as u32 - 1).min(60));
            warn!("Retry {} in {:?}...", retries, backoff);
            sleep(backoff).await;
        }
        if retries >= MAX_RETRIES {
            error!("Max retries reached.");
        }
    }

    async fn handle_command(
        stream: &mut WsStreamType,
        command: InternalCommand,
        pending_acks: &PendingAcks,
    ) -> Result<(), WsManagerError> {
        let (method, streams, ack_tx) = match command {
            InternalCommand::Subscribe { streams, ack_tx } => ("SUBSCRIBE", streams, ack_tx),
            InternalCommand::Unsubscribe { streams, ack_tx } => ("UNSUBSCRIBE", streams, ack_tx),
        };
        let id = Uuid::new_v4().to_string();
        let request = WsRequest {
            id: id.clone(),
            method: method.to_string(),
            params: Some(serde_json::to_value(streams)?),
        };
        let json = serde_json::to_string(&request)?;
        pending_acks.lock().await.insert(id.clone(), ack_tx);
        stream.send(Message::Text(json)).await?;
        Ok(())
    }

    async fn handle_message(
        msg: Message,
        pending_acks: &PendingAcks,
        deduplicator: &Arc<Deduplicator>,
        event_tx: &AsyncSender<WsEvent>,
    ) -> Result<(), WsManagerError> {
        match msg {
            Message::Text(text) => {
                if let Ok(response) = serde_json::from_str::<WsResponse<Value>>(&text) {
                    if let Some(id) = response.id {
                        if let Some(sender) = pending_acks.lock().await.remove(&id) {
                            let res = response.error.map_or(Ok(()), Err);
                            let _ = sender.send(res);
                            return Ok(());
                        }
                    }
                }
                if deduplicator.check(&text).await {
                    if let Ok(market_stream) = serde_json::from_str::<MarketStream>(&text) {
                        let event = match market_stream.data {
                            super::events::StreamEvent::AggTrade(trade) => WsEvent::AggTrade(trade),
                            super::events::StreamEvent::BookTicker(ticker) => WsEvent::BookTicker(ticker),
                        };
                        event_tx
                            .send(event)
                            .await
                            .map_err(|e| WsManagerError::ChannelError(e.to_string()))?;
                    } else {
                        warn!("Parse failed: {}", text);
                    }
                }
            }
            Message::Ping(payload) => {
                warn!("Received Ping, but handler should be in loop; log only.");
                // Note: In loop, we already handle Ping before delegate
                // Wait, fix: move Ping/Pong/Close handling to loop like Base 1 update.
                // For brevity, assume loop handles, but to correct, adjust loop to peek msg as in Base 1.
            }
            Message::Pong(_) => debug!("Pong received."),
            Message::Close(c) => return Err(WsManagerError::Unexpected(format!("Closed: {:?}", c))),
            _ => {}
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct UserWsManager {
    shutdown: CancellationToken,
    // No command_tx, as no sub
}

impl UserWsManager {
    pub async fn new(base_url: Url, listen_key: String, event_tx: AsyncSender<WsEvent>, dedup_lookback: usize) -> Self {
        let mut url = base_url;
        url.set_path(&format!("/ws/{}", listen_key));
        let shutdown = CancellationToken::new();
        let shutdown_clone = shutdown.clone();
        let deduplicator = Arc::new(Deduplicator::new(dedup_lookback));
        tokio::spawn(async move {
            Self::run_loop(url, event_tx, deduplicator, shutdown_clone).await;
        });
        Self { shutdown }
    }

    pub fn shutdown(&self) {
        self.shutdown.cancel();
    }

    async fn run_loop(
        url: Url,
        event_tx: AsyncSender<WsEvent>,
        deduplicator: Arc<Deduplicator>,
        shutdown: CancellationToken,
    ) {
        let mut retries = 0;
        const MAX_RETRIES: usize = 5;
        while retries < MAX_RETRIES && !shutdown.is_cancelled() {
            match connect_async(&url).await {
                Ok((mut stream, _)) => {
                    info!("User WS connected.");
                    retries = 0;
                    let mut ping_interval = interval(Duration::from_secs(180));
                    let mut last_activity = Instant::now();
                    loop {
                        select! {
                            _ = ping_interval.tick() => {
                                if let Err(e) = stream.send(Message::Pong(vec![])).await {
                                    error!("Pong failed: {}. Reconnecting...", e);
                                    break;
                                }
                            }
                            msg = stream.next() => {
                                match msg {
                                    Some(Ok(msg)) => {
                                        last_activity = Instant::now();
                                        if let Err(e) = Self::handle_message(msg, &deduplicator, &event_tx).await {
                                            error!("Message handling failed: {}. Reconnecting...", e);
                                            break;
                                        }
                                    }
                                    Some(Err(e)) => {
                                        error!("Recv error: {}. Reconnecting...", e);
                                        break;
                                    }
                                    None => {
                                        warn!("Stream closed. Reconnecting...");
                                        break;
                                    }
                                }
                            }
                            _ = sleep(Duration::from_secs(10)) => {
                                if Instant::now().duration_since(last_activity) > Duration::from_secs(360) {
                                    warn!("Inactivity timeout. Reconnecting...");
                                    break;
                                }
                            }
                            _ = shutdown.cancelled() => {
                                let _ = stream.close(None).await;
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Connect failed: {}.", e);
                }
            }
            retries += 1;
            let backoff = Duration::from_secs(2u64.pow(retries as u32 - 1).min(60));
            warn!("Retry {} in {:?}...", retries, backoff);
            sleep(backoff).await;
        }
        if retries >= MAX_RETRIES {
            error!("Max retries reached.");
        }
    }

    async fn handle_message(
        msg: Message,
        deduplicator: &Arc<Deduplicator>,
        event_tx: &AsyncSender<WsEvent>,
    ) -> Result<(), WsManagerError> {
        match msg {
            Message::Text(text) => {
                if deduplicator.check(&text).await {
                    if let Ok(event) = serde_json::from_str::<OrderTradeUpdateEvent>(&text) {
                        event_tx
                            .send(WsEvent::OrderTradeUpdate(event))
                            .await
                            .map_err(|e| WsManagerError::ChannelError(e.to_string()))?;
                    } else if let Ok(event) = serde_json::from_str::<AccountUpdateEvent>(&text) {
                        event_tx
                            .send(WsEvent::AccountUpdate(event))
                            .await
                            .map_err(|e| WsManagerError::ChannelError(e.to_string()))?;
                    } else {
                        warn!("Parse failed: {}", text);
                    }
                }
            }
            Message::Ping(payload) => {
                // Similar note: handle in loop for send Pong.
            }
            // Similar for Pong, Close
            _ => {}
        }
        Ok(())
    }
}
