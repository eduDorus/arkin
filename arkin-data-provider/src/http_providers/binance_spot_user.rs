use std::sync::Arc;

use anyhow::Result;
use arkin_core::prelude::*;
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use reqwest::{Client, Request};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use time::UtcDateTime;
use typed_builder::TypedBuilder;

use crate::errors::ProviderError;
use crate::http::{HttpRequest, HttpRequestContext};
use crate::traits::HttpProvider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotAccountInfo {
    pub maker_commission: u32,
    pub taker_commission: u32,
    pub buyer_commission: u32,
    pub seller_commission: u32,
    pub commission_rates: CommissionRates,
    pub can_trade: bool,
    pub can_withdraw: bool,
    pub can_deposit: bool,
    pub brokered: bool,
    pub require_self_trade_prevention: bool,
    pub prevent_sor: bool,
    pub update_time: u64,
    pub account_type: String,
    pub balances: Vec<SpotAccountBalance>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommissionRates {
    pub maker: String,
    pub taker: String,
    pub buyer: String,
    pub seller: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotAccountBalance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotOrderResponse {
    pub symbol: String,
    pub order_id: u64,
    pub order_list_id: i64,
    pub client_order_id: String,
    pub price: String,
    pub orig_qty: String,
    pub executed_qty: String,
    pub cummulative_quote_qty: String,
    pub status: String,
    pub time_in_force: String,
    pub order_type: String,
    pub side: String,
    pub stop_price: String,
    pub iceberg_qty: String,
    pub time: u64,
    pub update_time: u64,
    pub is_working: bool,
    pub orig_quote_order_qty: String,
}

#[derive(TypedBuilder)]
pub struct BinanceSpotUserHttpProvider {
    pub api_key: String,
    pub api_secret: String,
    pub http_url: String,
    pub persistence: Arc<dyn PersistenceReader>,
}

#[async_trait]
impl HttpProvider for BinanceSpotUserHttpProvider {
    fn get_endpoints(&self) -> Vec<HttpRequest> {
        vec![
            HttpRequest::new_polling(
                HttpRequestContext {
                    channel: Channel::OrderBook, // Account info
                    method: reqwest::Method::GET,
                    endpoint: "/api/v3/account".to_string(),
                    params: std::collections::BTreeMap::new(),
                    is_signed: true,
                    custom_headers: None,
                    last_fetched: Arc::new(std::sync::atomic::AtomicU64::new(0)),
                },
                std::time::Duration::from_secs(300), // 5 minutes for account info
            ),
            HttpRequest::new_polling(
                HttpRequestContext {
                    channel: Channel::Trades, // Orders
                    method: reqwest::Method::GET,
                    endpoint: "/api/v3/allOrders".to_string(),
                    params: std::collections::BTreeMap::new(),
                    is_signed: true,
                    custom_headers: None,
                    last_fetched: Arc::new(std::sync::atomic::AtomicU64::new(0)),
                },
                std::time::Duration::from_secs(60), // 1 minute for orders
            ),
        ]
    }

    fn build_request(&self, endpoint: &HttpRequestContext) -> Result<Request, ProviderError> {
        let timestamp = UtcDateTime::now().unix_timestamp() as u64 * 1000;
        let mut params = vec![("timestamp".to_string(), timestamp.to_string())];
        // Add other params from context
        for (k, v) in &endpoint.params {
            params.push((k.clone(), v.to_string()));
        }

        let query_string = params.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join("&");
        let signature = self.sign(&query_string);

        let url = format!(
            "{}{}?{}&signature={}",
            self.http_url, endpoint.endpoint, query_string, signature
        );

        let mut request_builder = Client::new()
            .request(endpoint.method.clone(), &url)
            .header("X-MBX-APIKEY", &self.api_key)
            .header("Content-Type", "application/json")
            .header("User-Agent", "arkin-data-provider/1.0")
            .header("Accept-Encoding", "gzip, deflate, br");

        // Add custom headers if provided
        if let Some(custom_headers) = &endpoint.custom_headers {
            for (key, value) in custom_headers {
                request_builder = request_builder.header(key, value);
            }
        }

        Ok(request_builder.build()?)
    }

    async fn parse(&self, _headers: &reqwest::header::HeaderMap, body: &str, channel: &Channel) -> Option<Event> {
        match channel {
            Channel::OrderBook => {
                // This is account info
                let account: SpotAccountInfo = serde_json::from_str(body).ok()?;
                let event_time = UtcDateTime::from_unix_timestamp((account.update_time / 1000) as i64).unwrap();
                let venue = self
                    .persistence
                    .get_venue(&VenueQuery::builder().name(VenueName::Binance).build())
                    .await
                    .ok()?;
                let account_type = AccountType::Spot;
                let mut balances = Vec::new();
                for b in account.balances {
                    if let Some(asset) = self
                        .persistence
                        .get_asset(&AssetQuery::builder().symbol(b.asset).build())
                        .await
                        .ok()
                    {
                        let free: Decimal = b.free.parse().unwrap_or_default();
                        let locked: Decimal = b.locked.parse().unwrap_or_default();
                        let total = free + locked;
                        if total > Decimal::ZERO {
                            balances.push(
                                BalanceUpdate::builder()
                                    .event_time(event_time)
                                    .venue(venue.clone())
                                    .account_type(account_type)
                                    .asset(asset)
                                    .quantity_change(Decimal::ZERO)
                                    .quantity(total)
                                    .build(),
                            );
                        }
                    }
                }
                let account_update = VenueAccountUpdate::builder()
                    .event_time(event_time)
                    .venue(venue)
                    .balances(balances)
                    .positions(Vec::new())
                    .reason("ACCOUNT_INFO".to_string())
                    .build();
                Some(Event::VenueAccountUpdate(Arc::new(account_update)))
            }
            Channel::Trades => {
                // This is all orders - for now just log
                let orders: Vec<SpotOrderResponse> = serde_json::from_str(body).ok()?;
                tracing::info!("Spot orders: {:?}", orders);
                None
            }
            _ => None,
        }
    }
}

impl BinanceSpotUserHttpProvider {
    fn sign(&self, query_string: &str) -> String {
        let mut mac =
            Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes()).expect("HMAC key initialization failed");
        mac.update(query_string.as_bytes());
        let result = mac.finalize().into_bytes();
        hex::encode(result)
    }
}
