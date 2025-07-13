use arkin_core::prelude::Deduplicator;
use async_tungstenite::{
    stream::Stream,
    tokio::{connect_async, TokioAdapter},
    tungstenite::Message,
    WebSocketStream,
};
use futures::StreamExt;
use kanal::{AsyncReceiver, AsyncSender};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::{
    net::TcpStream,
    select,
    sync::Semaphore,
    time::{interval, sleep},
};
use tokio_rustls::client::TlsStream;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, error};
use url::Url;

use crate::binance_fapi::BinanceSwapEvent;

#[derive(Error, Debug)]
pub enum WebsocketError {
    #[error("Channel send error: {0}")]
    ChannelSendError(#[from] kanal::SendError),

    #[error("Channel receive error: {0}")]
    ChannelReceiveError(#[from] kanal::ReceiveError),

    #[error("Websocket error: {0}")]
    WebSocketError(#[from] async_tungstenite::tungstenite::Error),

    #[error("Acuireing the lock failed: {0}")]
    LockError(#[from] tokio::sync::AcquireError),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),

    #[error("Error in configuration: {0}")]
    ConfigError(String),
}

pub enum AuthStrategy {
    None,
    LoginMsg(Box<dyn Fn() -> Message + Send + Sync>),
    UrlParam(Box<dyn Fn() -> String + Send + Sync>),
}

pub trait WSConfig {
    type Inbound: DeserializeOwned + Send + Sync + 'static;
    fn url(&self) -> Url;
    fn auth_strategy(&self) -> AuthStrategy;
    fn initial_subscribe(&self) -> Option<Message>;
    fn ping_interval(&self) -> Duration {
        Duration::from_secs(30)
    }
    fn format_ping(&self) -> Message {
        Message::Ping(Vec::new().into())
    }
    fn parse_inbound(&self, msg: &str) -> Result<Self::Inbound, WebsocketError>;
    // Add format_outbound if needed for market data, but for now assume subscribe in handler.
}

pub trait ExchangeMarketData: WSConfig {
    fn initial_subscribe(&self) -> Option<Message>; // e.g., subscribe to streams
}

pub trait ExchangeTrading: WSConfig {
    type Outbound: Send + Sync + 'static;
    fn format_outbound(&self, out: &Self::Outbound) -> Message;
}

// Base manager, generic over C: WSConfig
struct WSManager<C: WSConfig + Send + Sync + 'static> {
    config: Arc<C>,
    deduplicator: Arc<Deduplicator>,
    limit_connections: Arc<Semaphore>,
    max_retries: usize,
}

impl<C: WSConfig + Send + Sync + 'static> WSManager<C> {
    fn new(config: C, connections: usize, dedup_lookback: usize, max_retries: usize) -> Self {
        Self {
            config: Arc::new(config),
            deduplicator: Deduplicator::new(dedup_lookback).into(),
            limit_connections: Arc::new(Semaphore::new(connections)),
            max_retries,
        }
    }

    async fn run(
        &mut self,
        inbound_tx: AsyncSender<C::Inbound>,
        shutdown: CancellationToken,
    ) -> Result<(), WebsocketError> {
        let websocket_tracker = TaskTracker::new();
        loop {
            select! {
                permit = self.limit_connections.clone().acquire_owned(), if !shutdown.is_cancelled() => {
                    let permit = permit?;
                    let config = self.config.clone();
                    let inbound_tx_clone = inbound_tx.clone();
                    let shutdown_clone = shutdown.clone();
                    let max_retries = self.max_retries;
                    let dedup = self.deduplicator.clone();
                    websocket_tracker.spawn(async move {
                        let mut retries = 0;
                        while retries < max_retries {
                            match Handler::new(config.as_ref(), inbound_tx_clone.clone(), shutdown_clone.clone(), dedup.clone()).await {
                                Ok(mut handler) => {
                                    if let Err(err) = handler.run().await {
                                        error!("Handler error: {:?}. Retrying...", err);
                                        retries += 1;
                                        let backoff = Duration::from_secs(2u64.pow(retries as u32 - 1));
                                        sleep(std::cmp::min(backoff, Duration::from_secs(60))).await;
                                    } else {
                                        break;  // Clean exit
                                    }
                                }
                                Err(err) => {
                                    error!("Connect failed: {:?}", err);
                                    retries += 1;
                                    sleep(Duration::from_secs(1)).await;
                                }
                            }
                            if shutdown_clone.is_cancelled() { break; }
                        }
                        if retries >= max_retries { error!("Max retries reached"); }
                        drop(permit);
                    });
                }
                _ = shutdown.cancelled() => {
                    websocket_tracker.close();
                    websocket_tracker.wait().await;
                    break;
                }
            }
        }
        Ok(())
    }
}

// Handler for base logic
struct Handler<'a, C: WSConfig> {
    config: &'a C,
    stream: WebSocketStream<Stream<TokioAdapter<TcpStream>, TokioAdapter<TlsStream<TcpStream>>>>,
    inbound_tx: AsyncSender<C::Inbound>,
    shutdown: CancellationToken,
    deduplicator: Arc<Deduplicator>, // Added for use
}

impl<'a, C: WSConfig> Handler<'a, C> {
    async fn new(
        config: &'a C,
        inbound_tx: AsyncSender<C::Inbound>,
        shutdown: CancellationToken,
        deduplicator: Arc<Deduplicator>, // Pass from manager
    ) -> Result<Self, WebsocketError> {
        let mut url = config.url();
        if let AuthStrategy::UrlParam(f) = config.auth_strategy() {
            url.set_path(&format!("/ws/{}", f()));
        }
        let (mut stream, _) = connect_async(url.to_string()).await.map_err(WebsocketError::from)?;

        // Send auth if msg
        if let AuthStrategy::LoginMsg(f) = &config.auth_strategy() {
            stream.send(f()).await.map_err(WebsocketError::from)?;
        }

        // Send initial subscribe if provided (for market data, None for Binance public)
        if let Some(sub) = config.initial_subscribe() {
            stream.send(sub).await.map_err(WebsocketError::from)?;
        }

        Ok(Self {
            config,
            stream,
            inbound_tx,
            shutdown,
            deduplicator,
        })
    }

    async fn run(&mut self) -> Result<(), WebsocketError> {
        let mut ping_ticker = interval(self.config.ping_interval());
        loop {
            select! {
                _ = ping_ticker.tick() => {
                    self.stream.send(self.config.format_ping()).await?;
                }
                Some(msg) = self.stream.next() => {
                    let msg = msg.map_err(WebsocketError::from)?;
                    match msg {
                        Message::Text(text) => {
                            if self.deduplicator.check(&text).await {  // Dedup before parse
                                let parsed = self.config.parse_inbound(&text)?;
                                self.inbound_tx.send(parsed).await.map_err(WebsocketError::from)?;
                            }
                        }
                        Message::Ping(p) => {
                            self.stream.send(Message::Pong(p)).await?;
                        }
                        Message::Close(_) => {
                            return Err(WebsocketError::UnexpectedError("Connection closed by server".into()));
                        }
                        _ => {}  // Ignore binary, etc.
                    }
                }
                _ = self.shutdown.cancelled() => {
                    self.stream.close(None).await?;
                    break;
                }
            }
        }
        Ok(())
    }
}

// MarketData specific
pub struct MarketDataWSManager<C: ExchangeMarketData + Send + Sync + 'static>(WSManager<C>);

impl<C: ExchangeMarketData + Send + Sync + 'static> MarketDataWSManager<C> {
    pub fn new(config: C, connections: usize, dedup_lookback: usize) -> Self {
        Self(WSManager::new(config, connections, dedup_lookback, 5))
    }

    pub async fn run(
        &mut self,
        inbound_tx: AsyncSender<C::Inbound>,
        shutdown: CancellationToken,
    ) -> Result<(), WebsocketError> {
        // In handler new, after auth, send initial_subscribe
        // But for simplicity, assume handler sends it post-connect.
        // Extend Handler for market: in run, send subscribe once.
        self.0.run(inbound_tx, shutdown).await // Placeholder; override if needed.
    }
}

// Trading specific
pub struct TradingWSManager<C: ExchangeTrading + Send + Sync + 'static> {
    inner: WSManager<C>,
    outbound_rx: AsyncReceiver<C::Outbound>,
}

impl<C: ExchangeTrading + Send + Sync + 'static> TradingWSManager<C> {
    pub fn new(config: C, connections: usize, dedup_lookback: usize, outbound_rx: AsyncReceiver<C::Outbound>) -> Self {
        Self {
            inner: WSManager::new(config, connections, dedup_lookback, 5),
            outbound_rx,
        }
    }

    pub async fn run(
        &mut self,
        inbound_tx: AsyncSender<C::Inbound>,
        shutdown: CancellationToken,
    ) -> Result<(), WebsocketError> {
        // In handler run, add select on outbound_rx, send formatted.
        // Need to extend Handler to take outbound_rx, loop send when recv.
        self.inner.run(inbound_tx, shutdown).await // Placeholder; impl extension.
    }
}

// Binance impl
pub struct BinanceMarketData {
    streams: Vec<String>, // e.g., vec!["btcusdt@aggTrade", "ethusdt@bookTicker"]
}

#[derive(Serialize, Clone)]
pub struct BinanceSubscription {
    method: String,
    params: Vec<String>,
    id: u64,
}

impl BinanceSubscription {
    pub fn new(channels: &[String]) -> Self {
        Self {
            method: "SUBSCRIBE".to_string(),
            params: channels.iter().map(|c| c.to_string()).collect(),
            id: 0,
        }
    }

    pub fn update_id(&mut self, id: u64) {
        self.id = id;
    }
}

impl From<BinanceSubscription> for Message {
    fn from(sub: BinanceSubscription) -> Self {
        Message::Text(serde_json::to_string(&sub).expect("Failed to serialize subscription").into())
    }
}

impl WSConfig for BinanceMarketData {
    type Inbound = BinanceSwapEvent;

    fn url(&self) -> Url {
        Url::parse("wss://fstream.binance.com/ws").expect("Invalid base URL")
    }

    fn initial_subscribe(&self) -> Option<Message> {
        let sub = BinanceSubscription::new(&self.streams);
        Some(sub.into())
    }

    fn auth_strategy(&self) -> AuthStrategy {
        AuthStrategy::None
    }

    fn ping_interval(&self) -> Duration {
        Duration::from_secs(300) // 5min for keep-alive pongs
    }

    fn format_ping(&self) -> Message {
        Message::Pong(Vec::new().into()) // Unsolicited pong as per docs
    }

    fn parse_inbound(&self, msg: &str) -> Result<Self::Inbound, WebsocketError> {
        debug!(target: "ws", "received msg from ws: {}", msg);
        serde_json::from_str(msg).map_err(|e| WebsocketError::UnexpectedError(e.to_string()))
    }
}

impl ExchangeMarketData for BinanceMarketData {
    fn initial_subscribe(&self) -> Option<Message> {
        None // No JSON subscribe needed
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::timeout;
    use tracing::info;

    use super::*;

    #[tokio::test]
    #[test_log::test]
    async fn subscribe_binance_agg_trades() {
        let config = BinanceMarketData {
            streams: vec!["btcusdt@aggTrade".to_string()],
        };
        let mut manager = MarketDataWSManager::new(config, 1, 100); // 1 conn
        let (inbound_tx, inbound_rx) = kanal::unbounded_async::<BinanceSwapEvent>();
        let shutdown = CancellationToken::new();

        let shutdown_token = shutdown.clone();
        let handle = tokio::spawn(async move {
            manager.run(inbound_tx, shutdown_token.clone()).await.unwrap();
        });

        // Wait for some data
        while let Ok(Ok(event)) = timeout(Duration::from_secs(5), inbound_rx.recv()).await {
            match event {
                BinanceSwapEvent::AggTrade(data) => {
                    info!(target: "ws", "Received: {}", data);
                    assert_eq!(data.event_type, "aggTrade");
                    break;
                }
                BinanceSwapEvent::SubscribeResponse(_) => info!(target: "ws", "received subscribe confirmation"),
                e => panic!("received unknow message: {}", e),
            }
        }

        shutdown.cancel();
        handle.await.unwrap();
    }

    #[tokio::test]
    #[test_log::test]
    async fn subscribe_binance_ticks() {
        let config = BinanceMarketData {
            streams: vec!["btcusdt@bookTicker".to_string()],
        };
        let mut manager = MarketDataWSManager::new(config, 1, 100);
        let (inbound_tx, inbound_rx) = kanal::unbounded_async::<BinanceSwapEvent>();
        let shutdown = CancellationToken::new();

        let shutdown_token = shutdown.clone();
        let handle = tokio::spawn(async move {
            manager.run(inbound_tx, shutdown_token.clone()).await.unwrap();
        });

        while let Ok(Ok(event)) = timeout(Duration::from_secs(5), inbound_rx.recv()).await {
            match event {
                BinanceSwapEvent::Tick(data) => {
                    info!(target: "ws", "Received: {}", data);
                    assert_eq!(data.event_type, "bookTicker");
                    break;
                }
                BinanceSwapEvent::SubscribeResponse(_) => info!(target: "ws", "received subscribe confirmation"),
                e => panic!("received unknow message: {}", e),
            }
        }

        shutdown.cancel();
        handle.await.unwrap();
    }
}
