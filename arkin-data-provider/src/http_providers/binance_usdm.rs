use std::sync::Arc;

use anyhow::Result;
use arkin_core::prelude::*;
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use reqwest::{Client, Request};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use time::UtcDateTime;
use typed_builder::TypedBuilder;

use crate::http::{HttpRequest, HttpRequestContext};
use crate::{HttpProvider, ProviderError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllOrdersResponse {
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
pub struct BinanceUsdmUserHttpProvider {
    pub api_key: String,
    pub api_secret: String,
    pub http_url: String,
    pub persistence: Arc<dyn PersistenceReader>,
}

#[async_trait]
impl HttpProvider for BinanceUsdmUserHttpProvider {
    fn get_endpoints(&self) -> Vec<HttpRequest> {
        vec![HttpRequest::new_polling(
            HttpRequestContext {
                channel: Channel::Trades,
                method: reqwest::Method::GET,
                endpoint: "/fapi/v1/allOrders".to_string(),
                params: std::collections::BTreeMap::new(),
                is_signed: true,
                custom_headers: None,
                last_fetched: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            },
            std::time::Duration::from_secs(60),
        )]
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

    async fn parse(&self, _headers: &reqwest::header::HeaderMap, body: &str, _channel: &Channel) -> Option<Event> {
        let orders: Vec<AllOrdersResponse> = serde_json::from_str(body).ok()?;
        // For now, just log
        tracing::info!("All orders: {:?}", orders);
        None
    }
}

impl BinanceUsdmUserHttpProvider {
    fn sign(&self, query_string: &str) -> String {
        let mut mac =
            Hmac::<Sha256>::new_from_slice(self.api_secret.as_bytes()).expect("HMAC key initialization failed");
        mac.update(query_string.as_bytes());
        let result = mac.finalize().into_bytes();
        hex::encode(result)
    }
}
