use std::sync::Arc;

use anyhow::Result;
use arkin_core::prelude::*;
use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;
use tokio::sync::RwLock;
use tracing::{info, warn};
use typed_builder::TypedBuilder;
use url::Url;
use uuid::Uuid;

use crate::errors::ProviderError;
use crate::traits::WebSocketProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenKeyResponse {
    #[serde(rename = "listenKey")]
    pub listen_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "e")]
pub enum SpotUserDataEvent {
    #[serde(rename = "outboundAccountPosition")]
    OutboundAccountPosition(OutboundAccountPositionEvent),
    #[serde(rename = "balanceUpdate")]
    BalanceUpdate(BalanceUpdateEvent),
    #[serde(rename = "executionReport")]
    ExecutionReport(ExecutionReportEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundAccountPositionEvent {
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "u")]
    pub last_account_update: u64,
    #[serde(rename = "B")]
    pub balances: Vec<SpotBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotBalance {
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "f")]
    pub free: Decimal,
    #[serde(rename = "l")]
    pub locked: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceUpdateEvent {
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "d")]
    pub balance_delta: Decimal,
    #[serde(rename = "T")]
    pub clear_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReportEvent {
    #[serde(rename = "E")]
    pub event_time: u64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "c")]
    pub client_order_id: String,
    #[serde(rename = "S")]
    pub side: String,
    #[serde(rename = "o")]
    pub order_type: String,
    #[serde(rename = "f")]
    pub time_in_force: String,
    #[serde(rename = "q")]
    pub quantity: Decimal,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "P")]
    pub stop_price: Decimal,
    #[serde(rename = "F")]
    pub iceberg_quantity: Decimal,
    #[serde(rename = "g")]
    pub order_list_id: i64,
    #[serde(rename = "C")]
    pub original_client_order_id: Option<String>,
    #[serde(rename = "x")]
    pub current_execution_type: String,
    #[serde(rename = "X")]
    pub current_order_status: String,
    #[serde(rename = "r")]
    pub order_reject_reason: String,
    #[serde(rename = "i")]
    pub order_id: u64,
    #[serde(rename = "l")]
    pub last_executed_quantity: Decimal,
    #[serde(rename = "z")]
    pub cumulative_filled_quantity: Decimal,
    #[serde(rename = "L")]
    pub last_executed_price: Decimal,
    #[serde(rename = "n")]
    pub commission_amount: Decimal,
    #[serde(rename = "N")]
    pub commission_asset: Option<String>,
    #[serde(rename = "T")]
    pub transaction_time: u64,
    #[serde(rename = "t")]
    pub trade_id: i64,
    #[serde(rename = "I")]
    pub ignore: u64,
    #[serde(rename = "w")]
    pub is_order_working: bool,
    #[serde(rename = "m")]
    pub is_maker: bool,
    #[serde(rename = "M")]
    pub ignore2: bool,
    #[serde(rename = "O")]
    pub order_creation_time: u64,
    #[serde(rename = "Z")]
    pub cumulative_quote_asset_transacted_quantity: Decimal,
    #[serde(rename = "Y")]
    pub last_quote_asset_transacted_quantity: Decimal,
    #[serde(rename = "Q")]
    pub quote_order_quantity: Decimal,
    #[serde(rename = "W")]
    pub working_time: u64,
    #[serde(rename = "V")]
    pub self_trade_prevention_mode: String,
}

#[derive(TypedBuilder)]
pub struct BinanceSpotUserWsProvider {
    pub api_key: String,
    pub api_secret: String,
    pub http_url: Url,
    pub ws_url: Url,
    pub persistence: Arc<dyn PersistenceReader>,
    #[builder(default = Arc::new(RwLock::new(None)))]
    pub listen_key: Arc<RwLock<Option<String>>>,
}

#[async_trait]
impl WebSocketProvider for BinanceSpotUserWsProvider {
    fn name(&self) -> &str {
        "BinanceSpotUser"
    }

    fn url(&self) -> String {
        if let Ok(guard) = self.listen_key.try_read() {
            if let Some(listen_key) = guard.as_ref() {
                let mut url = self.ws_url.clone();
                url.path_segments_mut().expect("Invalid base URL").push(listen_key);
                return url.to_string();
            }
        }
        self.ws_url.to_string()
    }

    async fn setup(&self) -> Result<(), ProviderError> {
        let listen_key = self.start_listen_key().await?;
        *self.listen_key.write().await = Some(listen_key.clone());

        // Start keepalive task
        let api_key = self.api_key.clone();
        let http_url = self.http_url.clone();
        let listen_key = Arc::clone(&self.listen_key);
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1800)); // 30 minutes
            loop {
                interval.tick().await;
                if let Some(key) = listen_key.read().await.as_ref() {
                    if let Err(e) = keepalive_listen_key(&http_url, &api_key, key).await {
                        tracing::error!("Failed to keepalive listen key: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn teardown(&self) -> Result<(), ProviderError> {
        if let Some(listen_key) = self.listen_key.read().await.as_ref() {
            self.close_listen_key(listen_key).await?;
        }
        Ok(())
    }

    fn subscribe_msg(&self) -> Option<String> {
        None
    }

    async fn parse(&self, msg: &str) -> Result<Option<Event>, ProviderError> {
        info!("Received user data event: {:?}", msg);
        let event: SpotUserDataEvent = serde_json::from_str(msg).map_err(ProviderError::JsonParseError)?;
        match event {
            SpotUserDataEvent::OutboundAccountPosition(e) => {
                let event_time = UtcDateTime::from_unix_timestamp((e.event_time / 1000) as i64).unwrap();
                let venue = self
                    .persistence
                    .get_venue(&VenueQuery::builder().name(VenueName::Binance).build())
                    .await
                    .map_err(|e| ProviderError::PersistenceError(e.into()))?;
                let account_type = AccountType::Spot;
                let mut balances = Vec::new();
                for b in e.balances {
                    if let Ok(asset) = self.persistence.get_asset(&AssetQuery::builder().symbol(b.asset).build()).await
                    {
                        let total = b.free + b.locked;
                        balances.push(
                            BalanceUpdate::builder()
                                .event_time(event_time)
                                .venue(venue.clone())
                                .account_type(account_type)
                                .asset(asset)
                                .quantity_change(Decimal::ZERO) // Account position doesn't provide delta
                                .quantity(total)
                                .build(),
                        );
                    }
                }
                let account_update = VenueAccountUpdate::builder()
                    .event_time(event_time)
                    .venue(venue)
                    .balances(balances)
                    .positions(Vec::new()) // Spot doesn't have positions like futures
                    .reason("ACCOUNT_UPDATE".to_string())
                    .build();
                Ok(Some(Event::VenueAccountUpdate(Arc::new(account_update))))
            }
            SpotUserDataEvent::BalanceUpdate(e) => {
                let event_time = UtcDateTime::from_unix_timestamp((e.event_time / 1000) as i64).unwrap();
                let venue = self
                    .persistence
                    .get_venue(&VenueQuery::builder().name(VenueName::Binance).build())
                    .await
                    .map_err(|e| ProviderError::PersistenceError(e.into()))?;
                let account_type = AccountType::Spot;
                let mut balances = Vec::new();
                if let Ok(asset) = self.persistence.get_asset(&AssetQuery::builder().symbol(e.asset).build()).await {
                    balances.push(
                        BalanceUpdate::builder()
                            .event_time(event_time)
                            .venue(venue.clone())
                            .account_type(account_type)
                            .asset(asset)
                            .quantity_change(e.balance_delta)
                            .quantity(Decimal::ZERO) // Balance update doesn't provide total
                            .build(),
                    );
                }
                let account_update = VenueAccountUpdate::builder()
                    .event_time(event_time)
                    .venue(venue)
                    .balances(balances)
                    .positions(Vec::new())
                    .reason("BALANCE_UPDATE".to_string())
                    .build();
                Ok(Some(Event::VenueAccountUpdate(Arc::new(account_update))))
            }
            SpotUserDataEvent::ExecutionReport(e) => {
                let event_time = UtcDateTime::from_unix_timestamp((e.event_time / 1000) as i64).unwrap();

                // Parse client_order_id as the update ID
                let id_str = if e.client_order_id.starts_with("web_") {
                    &e.client_order_id[4..]
                } else {
                    &e.client_order_id
                };
                let id = match Uuid::parse_str(id_str) {
                    Ok(uuid) => uuid,
                    Err(_) => {
                        warn!("Failed to parse client_order_id '{}' as UUID", e.client_order_id);
                        return Ok(None);
                    }
                };

                // Map status
                let status = match e.current_order_status.as_str() {
                    "NEW" => VenueOrderStatus::New,
                    "PARTIALLY_FILLED" => VenueOrderStatus::PartiallyFilled,
                    "FILLED" => VenueOrderStatus::Filled,
                    "CANCELED" => VenueOrderStatus::Cancelled,
                    "REJECTED" => VenueOrderStatus::Rejected,
                    "EXPIRED" => VenueOrderStatus::Expired,
                    _ => VenueOrderStatus::New,
                };

                // For spot, use the values from the event
                let commission_asset = if let Some(asset_symbol) = &e.commission_asset {
                    self.persistence
                        .get_asset(&AssetQuery::builder().symbol(asset_symbol.clone()).build())
                        .await
                        .ok()
                } else {
                    None
                };

                let update = VenueOrderUpdate::builder()
                    .id(id)
                    .event_time(event_time)
                    .status(status)
                    .filled_quantity(e.cumulative_filled_quantity)
                    .filled_price(e.last_executed_price) // Use last executed price as filled price
                    .last_filled_quantity(e.last_executed_quantity)
                    .last_filled_price(e.last_executed_price)
                    .commission(e.commission_amount)
                    .commission_asset(commission_asset)
                    .build();

                Ok(Some(Event::VenueOrderUpdate(Arc::new(update))))
            }
        }
    }
}

impl BinanceSpotUserWsProvider {
    async fn start_listen_key(&self) -> Result<String, ProviderError> {
        let client = Client::new();
        let req_url = self.http_url.join("/api/v3/userDataStream").unwrap();
        let response = client
            .post(&req_url.to_string())
            .header("X-MBX-APIKEY", &self.api_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::RequestBuildError(format!(
                "Failed to start listen key: status={}, body={}",
                status, body
            )));
        }

        let listen_key_resp: ListenKeyResponse = response.json().await?;
        Ok(listen_key_resp.listen_key)
    }

    async fn close_listen_key(&self, listen_key: &str) -> Result<(), ProviderError> {
        let client = Client::new();
        let req_url = self.http_url.join("/api/v3/userDataStream").unwrap();
        let response = client
            .delete(&req_url.to_string())
            .header("X-MBX-APIKEY", &self.api_key)
            .query(&[("listenKey", listen_key)])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ProviderError::RequestBuildError(format!(
                "Failed to close listen key: status={}, body={}",
                status, body
            )));
        }
        Ok(())
    }
}

async fn keepalive_listen_key(http_url: &Url, api_key: &str, listen_key: &str) -> Result<()> {
    let client = Client::new();
    let url = http_url.join("/api/v3/userDataStream").unwrap();
    let response = client
        .put(&url.to_string())
        .header("X-MBX-APIKEY", api_key)
        .query(&[("listenKey", listen_key)])
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Failed to keepalive listen key: status={}, body={}",
            status,
            body
        ));
    }
    Ok(())
}
