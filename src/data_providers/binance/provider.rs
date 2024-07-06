use flume::Sender;
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info, warn};
use url::Url;

use crate::{
    config::BinanceDataProviderConfig,
    data_providers::{binance::parser::BinanceParser, ws::WebSocketManager, DataProvider},
    models::MarketEvent,
};

#[derive(Clone)]
pub struct BinanceDataProvider {
    url: Url,
    channels: Vec<String>,
    api_key: Option<String>,
    api_secret: Option<String>,
    connections_per_manager: usize,
    duplicate_lookback: usize,
}

impl BinanceDataProvider {
    pub fn new(config: &BinanceDataProviderConfig) -> Self {
        Self {
            url: config.ws_url.parse().expect("Failed to parse ws binance URL"),
            channels: config.ws_channels.to_owned(),
            api_key: config.api_key.to_owned(),
            api_secret: config.api_secret.to_owned(),
            connections_per_manager: config.connections_per_manager,
            duplicate_lookback: config.duplicate_lookback,
        }
    }
}

impl DataProvider for BinanceDataProvider {
    async fn start(&self, sender: Sender<MarketEvent>) {
        info!("Starting Binance data provider");

        // Check for API key and secret
        if self.api_key.is_none() || self.api_secret.is_none() {
            warn!("API key and secret are required for faster connection on Binance data provider");
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
                    let res = BinanceParser::parse(&data);
                    match res {
                        Ok(event) => {
                            sender.send_async(event).await.expect("Failed to send market event to sender");
                        }
                        Err(e) => error!("{}", e),
                    }
                }
                Err(e) => error!("{}", e),
            }
        }
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
