use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{debug, error, info, warn};
use typed_builder::TypedBuilder;
use url::Url;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;

use crate::swaps::BinanceSwapEvent;
use crate::ws::WebSocketManager;

#[derive(Serialize, Clone)]
pub struct Subscription {
    method: String,
    params: Vec<String>,
    id: u64,
}

impl Subscription {
    pub fn new(channels: Vec<&str>) -> Self {
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

impl From<Subscription> for Message {
    fn from(sub: Subscription) -> Self {
        Message::Text(serde_json::to_string(&sub).expect("Failed to serialize subscription").into())
    }
}

#[derive(TypedBuilder, Clone)]
pub struct BinanceIngestor {
    identifier: String,
    publisher: Arc<dyn Publisher>,
    persistence: Arc<Persistence>,
    url: Url,
    channels: Vec<String>,
    #[builder(default)]
    api_key: Option<String>,
    #[builder(default)]
    api_secret: Option<String>,
    #[builder(default = 1)]
    connections_per_manager: usize,
    #[builder(default = 10000)]
    duplicate_lookback: usize,
}

impl BinanceIngestor {
    async fn process_event(&self, data: String) {
        match serde_json::from_str::<BinanceSwapEvent>(&data) {
            Ok(e) => {
                debug!("BinanceSwapEvent: {}", e);
                if let Ok(instrument) = self.persistence.instrument_store.read_by_venue_symbol(&e.venue_symbol()).await
                {
                    debug!("Instrument found: {}", instrument.symbol);
                    match e {
                        BinanceSwapEvent::AggTrade(trade) => {
                            // "m": true: The buyer is the market maker.
                            // • The trade was initiated by a sell order from the taker.
                            // • The taker is selling, and the maker (buyer) is buying.
                            // "m": false: The seller is the market maker.
                            // • The trade was initiated by a buy order from the taker.
                            // • The taker is buying, and the maker (seller) is selling.
                            let side = if trade.maker {
                                MarketSide::Sell
                            } else {
                                MarketSide::Buy
                            };
                            let trade = AggTrade::new(
                                trade.event_time,
                                instrument,
                                trade.agg_trade_id,
                                side,
                                trade.price,
                                trade.quantity,
                            );
                            let trade = Arc::new(trade);
                            self.publisher.publish(trade).await;
                        }
                        BinanceSwapEvent::Tick(tick) => {
                            let tick = Tick::new(
                                tick.event_time,
                                instrument,
                                tick.update_id,
                                tick.bid_price,
                                tick.bid_quantity,
                                tick.ask_price,
                                tick.ask_quantity,
                            );
                            let tick = Arc::new(tick);
                            self.publisher.publish(tick).await;
                        }
                        _ => error!("type not impolemented"),
                    }
                } else {
                    warn!("Instrument not found for symbol: {}", e.venue_symbol());
                }
            }
            Err(e) => {
                error!("Failed to parse Binance event: {}", e);
                error!("Data: {}", data);
            }
        };
    }
}

#[async_trait]
impl Runnable for BinanceIngestor {
    fn identifier(&self) -> &str {
        &self.identifier
    }
    async fn start_tasks(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting binance ingestor...");

        // Check for API key and secret
        if self.api_key.is_none() || self.api_secret.is_none() {
            warn!("API key and secret are required for faster connection on Binance ingestor");
        }

        let mut ws_manager =
            WebSocketManager::new(self.url.clone(), self.connections_per_manager, self.duplicate_lookback);

        let (tx, rx) = kanal::unbounded_async();
        let subscription = Subscription::new(self.channels.iter().map(|c| c.as_str()).collect());

        let ws_manager_tracker = TaskTracker::new();
        let ws_manager_shutdown = shutdown.clone();
        ws_manager_tracker.spawn(async move {
            ws_manager.run(tx, subscription, ws_manager_shutdown).await.unwrap();
        });

        loop {
            tokio::select! {
                res = rx.recv() => {
                    match res {
                        Ok(data) => {
                            self.process_event(data).await;
                        }
                        Err(e) => {
                            error!("{}", e);
                            break;
                        }
                    }
                }
                _ = shutdown.cancelled() => {
                    info!("Shutting down binance ingestor...");
                    ws_manager_tracker.close();
                    ws_manager_tracker.wait().await;
                    break;
                }
            }
        }
        info!("Binance ingestor shutdown complete.");
        Ok(())
    }
}
