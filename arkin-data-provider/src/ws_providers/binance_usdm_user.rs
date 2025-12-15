use std::sync::Arc;

use anyhow::Result;
use arkin_core::prelude::*;
use async_trait::async_trait;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use time::UtcDateTime;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
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
pub enum UserDataEvent {
    #[serde(rename = "ORDER_TRADE_UPDATE")]
    OrderTradeUpdate(OrderTradeUpdateEvent),
    #[serde(rename = "ACCOUNT_UPDATE")]
    AccountUpdate(AccountUpdateEvent),
    #[serde(rename = "MARGIN_CALL")]
    MarginCall(MarginCallEvent),
    #[serde(rename = "TRADE_LITE")]
    TradeLite(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderTradeUpdateEvent {
    #[serde(rename = "T")]
    pub transaction_time: u64,
    #[serde(rename = "o")]
    pub order: OrderUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUpdateEvent {
    #[serde(rename = "T")]
    pub transaction_time: u64,
    #[serde(rename = "a")]
    pub update: AccountUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginCallEvent {
    #[serde(rename = "T")]
    pub transaction_time: u64,
    #[serde(rename = "p")]
    pub positions: Vec<BinancePositionUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdate {
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
    #[serde(rename = "ap")]
    pub average_price: Decimal,
    #[serde(rename = "sp")]
    pub stop_price: Decimal,
    #[serde(rename = "x")]
    pub execution_type: String,
    #[serde(rename = "X")]
    pub order_status: String,
    #[serde(rename = "i")]
    pub order_id: u64,
    #[serde(rename = "l")]
    pub last_filled_quantity: Decimal,
    #[serde(rename = "z")]
    pub cumulative_filled_quantity: Decimal,
    #[serde(rename = "L")]
    pub last_filled_price: Decimal,
    #[serde(rename = "N")]
    pub commission_asset: Option<String>,
    #[serde(rename = "n")]
    pub commission: Option<Decimal>,
    #[serde(rename = "T")]
    pub trade_time: u64,
    #[serde(rename = "t")]
    pub trade_id: u64,
    #[serde(rename = "b")]
    pub bids_notional: Decimal,
    #[serde(rename = "a")]
    pub asks_notional: Decimal,
    #[serde(rename = "m")]
    pub is_maker: bool,
    #[serde(rename = "R")]
    pub reduce_only: bool,
    #[serde(rename = "wt")]
    pub working_type: String,
    #[serde(rename = "ot")]
    pub original_order_type: String,
    #[serde(rename = "ps")]
    pub position_side: String,
    #[serde(rename = "cp")]
    pub close_position: bool,
    #[serde(rename = "AP")]
    pub activation_price: Option<Decimal>,
    #[serde(rename = "cr")]
    pub callback_rate: Option<Decimal>,
    #[serde(rename = "rp")]
    pub realized_profit: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountUpdate {
    #[serde(rename = "m")]
    pub reason: String,
    #[serde(rename = "B")]
    pub balances: Vec<BinanceBalanceUpdate>,
    #[serde(rename = "P")]
    pub positions: Vec<BinancePositionUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinanceBalanceUpdate {
    #[serde(rename = "a")]
    pub asset: String,
    #[serde(rename = "wb")]
    pub wallet_balance: Decimal,
    #[serde(rename = "cw")]
    pub cross_wallet_balance: Decimal,
    #[serde(rename = "bc")]
    pub balance_change: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinancePositionUpdate {
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "pa")]
    pub position_amount: Decimal,
    #[serde(rename = "ep")]
    pub entry_price: Decimal,
    #[serde(rename = "cr")]
    pub accumulated_realized: Decimal,
    #[serde(rename = "up")]
    pub unrealized_pnl: Decimal,
    #[serde(rename = "mt")]
    pub margin_type: String,
    #[serde(rename = "iw")]
    pub isolated_wallet: Decimal,
    #[serde(rename = "ps")]
    pub position_side: String,
}

#[derive(TypedBuilder)]
pub struct BinanceUsdmUserWsProvider {
    pub api_key: String,
    pub api_secret: String,
    pub http_url: Url,
    pub ws_url: Url,
    pub persistence: Arc<dyn PersistenceReader>,
    #[builder(default = Arc::new(RwLock::new(None)))]
    pub listen_key: Arc<RwLock<Option<String>>>,
}

#[async_trait]
impl WebSocketProvider for BinanceUsdmUserWsProvider {
    fn name(&self) -> &str {
        "BinanceUsdmUser"
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

    fn subscribe_msg(&self) -> Option<String> {
        None
    }

    async fn setup(&self) -> Result<(), ProviderError> {
        let listen_key = self.start_listen_key().await?;
        *self.listen_key.write().await = Some(listen_key.clone());

        // Start keepalive task
        let api_key = self.api_key.clone();
        let http_url = self.http_url.clone();
        let listen_key_arc = self.listen_key.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1800)); // 30 minutes
            loop {
                interval.tick().await;
                if let Some(key) = listen_key_arc.read().await.as_ref() {
                    if let Err(e) = keepalive_listen_key(&http_url, &api_key, key).await {
                        error!("Failed to keepalive listen key: {:?}", e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn teardown(&self) -> Result<(), ProviderError> {
        // if let Some(key) = self.listen_key.read().await.as_ref() {
        //     self.close_listen_key(key).await?;
        // }
        Ok(())
    }

    async fn parse(&self, msg: &str) -> Result<Option<Event>, ProviderError> {
        debug!("Parsing user data event: {:?}", msg);
        let event: UserDataEvent = serde_json::from_str(msg).map_err(ProviderError::JsonParseError)?;
        match event {
            UserDataEvent::OrderTradeUpdate(e) => {
                let event_time = UtcDateTime::from_unix_timestamp((e.transaction_time / 1000) as i64).unwrap();

                // Parse client_order_id as the update ID
                let id = match Uuid::parse_str(&e.order.client_order_id) {
                    Ok(uuid) => uuid,
                    Err(_) => {
                        warn!("Failed to parse client_order_id '{}' as UUID", e.order.client_order_id);
                        return Ok(None);
                    }
                };

                // Map status
                let status = match e.order.order_status.to_lowercase().as_str() {
                    "new" => VenueOrderStatus::New,
                    "partially_filled" => VenueOrderStatus::PartiallyFilled,
                    "filled" => VenueOrderStatus::Filled,
                    "canceled" => VenueOrderStatus::Cancelled,
                    "rejected" => VenueOrderStatus::Rejected,
                    "expired" => VenueOrderStatus::Expired,
                    _ => VenueOrderStatus::New,
                };

                let commission_asset = if let Some(asset_symbol) = &e.order.commission_asset {
                    self.persistence
                        .get_asset(&AssetQuery::builder().symbol(asset_symbol.clone()).build())
                        .await
                        .ok()
                } else {
                    None
                };

                println!("Building VenueOrderUpdate");
                println!("Building VenueOrderUpdate");
                let update = VenueOrderUpdate::builder()
                    .id(id)
                    .event_time(event_time)
                    .status(status)
                    .filled_quantity(e.order.cumulative_filled_quantity)
                    .filled_price(e.order.average_price)
                    .last_filled_quantity(e.order.last_filled_quantity)
                    .last_filled_price(e.order.last_filled_price)
                    .commission(e.order.commission.unwrap_or(Decimal::ZERO))
                    .commission_asset(commission_asset)
                    .build();
                println!("Built update: {:?}", update);
                println!("Built update: {:?}", update);
                info!("Parsed VenueOrderUpdate: {:?}", update);
                Ok(Some(Event::VenueOrderUpdate(Arc::new(update))))
            }
            UserDataEvent::TradeLite(_) => Ok(None),
            UserDataEvent::AccountUpdate(e) => {
                let event_time = UtcDateTime::from_unix_timestamp((e.transaction_time / 1000) as i64).unwrap();
                let venue = self
                    .persistence
                    .get_venue(&VenueQuery::builder().name(VenueName::Binance).build())
                    .await
                    .map_err(|e| ProviderError::PersistenceError(e.into()))?;
                let account_type = AccountType::Margin; // Futures
                let mut balances = Vec::new();
                for b in e.update.balances {
                    if let Ok(asset) = self.persistence.get_asset(&AssetQuery::builder().symbol(b.asset).build()).await
                    {
                        balances.push(
                            BalanceUpdate::builder()
                                .event_time(event_time)
                                .venue(venue.clone())
                                .account_type(account_type)
                                .asset(asset)
                                .quantity_change(b.balance_change)
                                .quantity(b.wallet_balance)
                                .build(),
                        );
                    }
                }
                let mut positions = Vec::new();
                for p in e.update.positions {
                    if let Ok(instrument) = self
                        .persistence
                        .get_instrument(&InstrumentQuery::builder().venue_symbol(p.symbol).build())
                        .await
                    {
                        let position_side = match p.position_side.as_str() {
                            "LONG" => PositionSide::Long,
                            "SHORT" => PositionSide::Short,
                            _ => continue,
                        };
                        positions.push(
                            PositionUpdate::builder()
                                .event_time(event_time)
                                .instrument(instrument)
                                .account_type(account_type)
                                .entry_price(p.entry_price)
                                .quantity(p.position_amount)
                                .realized_pnl(p.accumulated_realized)
                                .unrealized_pnl(p.unrealized_pnl)
                                .position_side(position_side)
                                .build(),
                        );
                    }
                }
                let account_update = VenueAccountUpdate::builder()
                    .event_time(event_time)
                    .venue(venue)
                    .balances(balances)
                    .positions(positions)
                    .reason(e.update.reason)
                    .build();
                Ok(Some(Event::VenueAccountUpdate(Arc::new(account_update))))
            }
            UserDataEvent::MarginCall(e) => {
                let event_time = UtcDateTime::from_unix_timestamp((e.transaction_time / 1000) as i64).unwrap();
                let venue = self
                    .persistence
                    .get_venue(&VenueQuery::builder().name(VenueName::Binance).build())
                    .await
                    .map_err(|e| ProviderError::PersistenceError(e.into()))?;
                let account_type = AccountType::Margin; // Futures
                let mut positions = Vec::new();
                for p in e.positions {
                    if let Ok(instrument) = self
                        .persistence
                        .get_instrument(&InstrumentQuery::builder().venue_symbol(p.symbol).build())
                        .await
                    {
                        let position_side = match p.position_side.as_str() {
                            "LONG" => PositionSide::Long,
                            "SHORT" => PositionSide::Short,
                            _ => continue,
                        };
                        positions.push(
                            PositionUpdate::builder()
                                .event_time(event_time)
                                .instrument(instrument)
                                .account_type(account_type)
                                .entry_price(p.entry_price)
                                .quantity(p.position_amount)
                                .realized_pnl(p.accumulated_realized)
                                .unrealized_pnl(p.unrealized_pnl)
                                .position_side(position_side)
                                .build(),
                        );
                    }
                }
                let account_update = VenueAccountUpdate::builder()
                    .event_time(event_time)
                    .venue(venue)
                    .balances(Vec::new()) // No balance changes in margin call
                    .positions(positions)
                    .reason("MARGIN_CALL".to_string())
                    .build();
                Ok(Some(Event::VenueAccountUpdate(Arc::new(account_update))))
            }
        }
    }
}

impl BinanceUsdmUserWsProvider {
    async fn start_listen_key(&self) -> Result<String, ProviderError> {
        let client = Client::new();
        info!("Http URL: {}", self.http_url);
        // Combine url
        let req_url = self.http_url.join("/fapi/v1/listenKey").unwrap();
        let request = client
            .post(&req_url.to_string())
            .header("X-MBX-APIKEY", &self.api_key)
            .header("Content-Type", "application/json")
            .header("User-Agent", "arkin-data-provider/1.0")
            .build()?;
        info!("Starting listen key with request: {:?}", request);
        let response = client.execute(request).await?;

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

    #[allow(dead_code)]
    async fn close_listen_key(&self, listen_key: &str) -> Result<(), ProviderError> {
        let client = Client::new();
        let url = self.http_url.join("/fapi/v1/listenKey").unwrap();
        let request = client
            .delete(url)
            .header("X-MBX-APIKEY", &self.api_key)
            .header("Content-Type", "application/json")
            .header("User-Agent", "arkin-data-provider/1.0")
            .query(&[("listenKey", listen_key)])
            .build()?;

        info!("Closing listen key with request: {:?}", request);
        let response = client.execute(request).await?;

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
    let url = http_url.join("/fapi/v1/listenKey").unwrap();
    let request = client
        .put(url)
        .header("X-MBX-APIKEY", api_key)
        .header("Content-Type", "application/json")
        .header("User-Agent", "arkin-data-provider/1.0")
        .query(&[("listenKey", listen_key)])
        .build()?;

    info!("Keeping alive listen key with request: {:?}", request);
    let response = client.execute(request).await?;

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
