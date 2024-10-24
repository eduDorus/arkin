use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use async_tungstenite::tungstenite::Message;
use serde::Serialize;
use tracing::{debug, error, info, warn};
use url::Url;

use arkin_core::prelude::*;
use arkin_persistance::prelude::*;

use crate::binance::swaps::BinanceSwapEvent;
use crate::config::BinanceIngestorConfig;
use crate::service::Ingestor;
use crate::ws::WebSocketManager;

#[derive(Clone)]
pub struct BinanceIngestor {
    persistance_service: Arc<PersistanceService>,
    url: Url,
    channels: Vec<String>,
    api_key: Option<String>,
    api_secret: Option<String>,
    connections_per_manager: usize,
    duplicate_lookback: usize,
}

impl BinanceIngestor {
    pub fn from_config(config: &BinanceIngestorConfig, persistance_service: Arc<PersistanceService>) -> Self {
        Self {
            persistance_service,
            url: config.ws_url.parse().expect("Failed to parse ws binance URL"),
            channels: config.ws_channels.to_owned(),
            api_key: config.api_key.to_owned(),
            api_secret: config.api_secret.to_owned(),
            connections_per_manager: config.connections_per_manager,
            duplicate_lookback: config.duplicate_lookback,
        }
    }
}

#[async_trait]
impl Ingestor for BinanceIngestor {
    async fn start(&self) -> Result<()> {
        info!("Starting binance ingestor...");

        // Check for API key and secret
        if self.api_key.is_none() || self.api_secret.is_none() {
            warn!("API key and secret are required for faster connection on Binance ingestor");
        }

        let mut ws_manager =
            WebSocketManager::new(self.url.clone(), self.connections_per_manager, self.duplicate_lookback);

        let (tx, rx) = flume::unbounded();
        let subscription = Subscription::new(self.channels.iter().map(|c| c.as_str()).collect());

        tokio::spawn(async move {
            ws_manager.run(tx, subscription).await.unwrap();
        });

        loop {
            let res = rx.recv_async().await;
            match res {
                Ok(data) => {
                    match serde_json::from_str::<BinanceSwapEvent>(&data) {
                        Ok(e) => {
                            info!("{}", e);
                            if let Ok(instrument) =
                                self.persistance_service.read_instrument_by_venue_symbol(e.venue_symbol()).await
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
                                        let trade = Trade::new(
                                            trade.event_time,
                                            instrument,
                                            trade.agg_trade_id,
                                            side,
                                            trade.price,
                                            trade.quantity,
                                        );
                                        if let Err(e) = self.persistance_service.insert_trade(trade).await {
                                            error!("Failed to insert trade: {}", e);
                                        }
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
                                        if let Err(e) = self.persistance_service.insert_tick(tick).await {
                                            error!("Failed to insert tick: {}", e);
                                        }
                                    }
                                }
                            } else {
                                continue;
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse Binance event: {}", e);
                            error!("Data: {}", data);
                            continue;
                        }
                    };
                }
                Err(e) => {
                    error!("{}", e);
                    break;
                }
            }
        }
        Ok(())
    }
}

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
        Message::Text(serde_json::to_string(&sub).expect("Failed to serialize subscription"))
    }
}
