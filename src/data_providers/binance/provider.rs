use flume::Sender;
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;
use tracing::info;
use url::Url;

use crate::{
    data_providers::{ws::WebSocketManager, DataProvider},
    models::MarketEvent,
};

pub struct BinanceDataProvider {}

impl Default for BinanceDataProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl BinanceDataProvider {
    pub fn new() -> Self {
        Self {}
    }
}

impl DataProvider for BinanceDataProvider {
    async fn start(&self, _sender: Sender<MarketEvent>) {
        println!("Starting Binance data provider");
        let url = Url::parse("wss://fstream.binance.com/ws").expect("Failed to parse ws binance URL");
        let mut ws_manager = WebSocketManager::new(url, 2, 100);

        let (tx, rx) = flume::unbounded();
        let subscription = Subscription::new(vec!["btcusdt@aggTrade", "btcusdt@ticker"]);

        ws_manager.run(tx, subscription).await.unwrap();

        while let Ok(event) = rx.recv_async().await {
            // Parse the event
            info!("Received event: {:?}", event);
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
