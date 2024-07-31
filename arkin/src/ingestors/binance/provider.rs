use std::sync::Arc;

use async_trait::async_trait;
use async_tungstenite::tungstenite::Message;
use serde::Serialize;
use tracing::{error, info, warn};
use url::Url;

use crate::{
    config::BinanceIngestorConfig,
    ingestors::{models::BinanceParser, ws::WebSocketManager, Ingestor},
    state::State,
};

#[derive(Clone)]
pub struct BinanceIngestor {
    state: Arc<State>,
    url: Url,
    channels: Vec<String>,
    api_key: Option<String>,
    api_secret: Option<String>,
    connections_per_manager: usize,
    duplicate_lookback: usize,
}

impl BinanceIngestor {
    pub fn new(state: Arc<State>, config: &BinanceIngestorConfig) -> Self {
        Self {
            state,
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
    async fn start(&self) {
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
                    let res = BinanceParser::parse_swap(&data);
                    match res {
                        Ok(event) => {
                            self.state.add_event(event);
                        }
                        Err(e) => error!("{}", e),
                    }
                }
                Err(e) => {
                    error!("{}", e);
                    break;
                }
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
