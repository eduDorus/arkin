use hmac::{Hmac, Mac};
use reqwest::Client;
use serde_urlencoded;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::debug;
use typed_builder::TypedBuilder;

use crate::config::BinanceUsdmExecutionConfig;
use crate::types::{BinanceCancelAllResponse, BinanceCancelResponse, BinanceOrderResponse, OrderParams};

type HmacSha256 = Hmac<Sha256>;

#[async_trait::async_trait]
pub trait BinanceUsdmRequest {
    fn endpoint(&self) -> &str;
    fn method(&self) -> reqwest::Method;
    fn parameters(&self) -> BTreeMap<String, serde_json::Value>;
    fn requires_signature(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone)]
pub struct BinanceUsdmClient {
    client: Client,
    api_key: String,
    api_secret: String,
    base_url: url::Url,
}

impl BinanceUsdmClient {
    pub fn new(config: BinanceUsdmExecutionConfig) -> Self {
        Self::new_with_client(config, None)
    }

    pub fn new_with_client(config: BinanceUsdmExecutionConfig, client: Option<Client>) -> Self {
        let client = client.unwrap_or_else(|| {
            Client::builder()
                .pool_max_idle_per_host(10) // Keep connections alive
                .pool_idle_timeout(std::time::Duration::from_secs(300)) // 5 minutes
                .build()
                .expect("Failed to build reqwest client")
        });

        Self {
            client,
            api_key: config.api_key,
            api_secret: config.api_secret,
            base_url: config.base_url,
        }
    }

    fn sign(&self, query_string: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes()).expect("HMAC can take key of any size");
        mac.update(query_string.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    pub async fn execute_request<R: serde::de::DeserializeOwned + Send + 'static>(
        &self,
        request: &impl BinanceUsdmRequest,
    ) -> Result<R, Box<dyn std::error::Error + Send + Sync>> {
        let url = self.base_url.join(&format!("fapi/v1{}", request.endpoint()))?;

        let mut params = request.parameters();

        // Add timestamp and signature if required
        if request.requires_signature() {
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
            params.insert("timestamp".to_string(), serde_json::Value::String(timestamp.to_string()));
            params.insert("recvWindow".to_string(), serde_json::Value::String("5000".to_string()));

            // Create URL-encoded query string for signing (parameters must be sorted alphabetically)
            let mut sign_params = std::collections::BTreeMap::new();
            for (key, value) in &params {
                if let Some(str_value) = value.as_str() {
                    sign_params.insert(key.clone(), str_value.to_string());
                }
            }
            let query_string = serde_urlencoded::to_string(&sign_params)?;

            let signature = self.sign(&query_string);
            params.insert("signature".to_string(), serde_json::Value::String(signature));
        }

        debug!("Executing USDM request to: {} with method: {}", url.as_str(), request.method());

        let mut req_builder = self.client.request(request.method(), url);

        // Set headers
        req_builder = req_builder.header("X-MBX-APIKEY", &self.api_key);
        req_builder = req_builder.header("Accept", "application/json");
        req_builder = req_builder.header("User-Agent", "Arkin-Binance-Client/1.0");
        req_builder = req_builder.header("Accept-Encoding", "gzip, deflate, br");

        // Handle parameters based on HTTP method
        match request.method() {
            reqwest::Method::GET => {
                // For GET requests, add all parameters as query parameters
                req_builder = req_builder.header("Content-Type", "application/x-www-form-urlencoded");
                for (key, value) in &params {
                    if let Some(str_value) = value.as_str() {
                        req_builder = req_builder.query(&[(key, str_value)]);
                    }
                }
            }
            reqwest::Method::POST | reqwest::Method::PUT | reqwest::Method::DELETE => {
                // For POST/PUT/DELETE requests, send ALL parameters as query parameters (including signature)
                req_builder = req_builder.header("Content-Type", "application/x-www-form-urlencoded");
                for (key, value) in &params {
                    if let Some(str_value) = value.as_str() {
                        req_builder = req_builder.query(&[(key, str_value)]);
                    }
                }
            }
            _ => {
                // For other methods, add as query parameters
                req_builder = req_builder.header("Content-Type", "application/x-www-form-urlencoded");
                for (key, value) in &params {
                    if let Some(str_value) = value.as_str() {
                        req_builder = req_builder.query(&[(key, str_value)]);
                    }
                }
            }
        }

        let response = req_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            tracing::error!("Binance USDM API error: {} - {}", status, body);
            return Err(format!("API error: {} - {}", status, body).into());
        }

        let result = response.json().await?;
        Ok(result)
    }

    pub async fn place_order(
        &self,
        params: OrderParams,
    ) -> Result<BinanceOrderResponse, Box<dyn std::error::Error + Send + Sync>> {
        let request = PlaceOrderRequest { params };
        self.execute_request(&request).await
    }

    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        client_order_id: Option<&str>,
    ) -> Result<BinanceCancelResponse, Box<dyn std::error::Error + Send + Sync>> {
        let request = CancelOrderRequest::builder()
            .symbol(symbol)
            .order_id(order_id)
            .client_order_id(client_order_id.map(|s| s.to_string()))
            .build();
        self.execute_request(&request).await
    }

    pub async fn cancel_all_orders(
        &self,
        symbol: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let request = CancelAllOrdersRequest {
            symbol: symbol.map(|s| s.to_string()),
        };
        let _response: BinanceCancelAllResponse = self.execute_request(&request).await?;
        Ok(())
    }
}

// Request implementations for USDM
pub struct PlaceOrderRequest {
    pub params: OrderParams,
}

#[async_trait::async_trait]
impl BinanceUsdmRequest for PlaceOrderRequest {
    fn endpoint(&self) -> &str {
        "/order"
    }

    fn method(&self) -> reqwest::Method {
        reqwest::Method::POST
    }

    fn parameters(&self) -> BTreeMap<String, serde_json::Value> {
        let mut params = BTreeMap::new();
        params.insert("symbol".to_string(), serde_json::Value::String(self.params.symbol.clone()));
        params.insert("side".to_string(), serde_json::Value::String(self.params.side.clone()));
        params.insert("type".to_string(), serde_json::Value::String(self.params.order_type.clone()));

        if let Some(ref tif) = self.params.time_in_force {
            params.insert("timeInForce".to_string(), serde_json::Value::String(tif.clone()));
        }
        if let Some(ref qty) = self.params.quantity {
            params.insert("quantity".to_string(), serde_json::Value::String(qty.clone()));
        }
        if let Some(ref quote_qty) = self.params.quote_order_qty {
            params.insert("quoteOrderQty".to_string(), serde_json::Value::String(quote_qty.clone()));
        }
        if let Some(ref price) = self.params.price {
            params.insert("price".to_string(), serde_json::Value::String(price.clone()));
        }
        if let Some(ref stop_price) = self.params.stop_price {
            params.insert("stopPrice".to_string(), serde_json::Value::String(stop_price.clone()));
        }
        if let Some(ref position_side) = self.params.position_side {
            params.insert("positionSide".to_string(), serde_json::Value::String(position_side.clone()));
        }
        if let Some(ref reduce_only) = self.params.reduce_only {
            params.insert("reduceOnly".to_string(), serde_json::Value::String(reduce_only.clone()));
        }
        if let Some(ref new_client_order_id) = self.params.new_client_order_id {
            params.insert(
                "newClientOrderId".to_string(),
                serde_json::Value::String(new_client_order_id.clone()),
            );
        }

        params
    }
}

#[derive(TypedBuilder)]
pub struct CancelOrderRequest {
    #[builder(setter(into))]
    pub symbol: String,
    #[builder(default)]
    pub order_id: Option<u64>,
    #[builder(default)]
    pub client_order_id: Option<String>,
}

#[async_trait::async_trait]
impl BinanceUsdmRequest for CancelOrderRequest {
    fn endpoint(&self) -> &str {
        "/order"
    }

    fn method(&self) -> reqwest::Method {
        reqwest::Method::DELETE
    }

    fn parameters(&self) -> BTreeMap<String, serde_json::Value> {
        let mut params = BTreeMap::new();
        params.insert("symbol".to_string(), serde_json::Value::String(self.symbol.clone()));

        if let Some(order_id) = self.order_id {
            params.insert("orderId".to_string(), serde_json::Value::String(order_id.to_string()));
        }
        if let Some(ref client_order_id) = self.client_order_id {
            params.insert(
                "origClientOrderId".to_string(),
                serde_json::Value::String(client_order_id.clone()),
            );
        }

        params
    }
}

pub struct CancelAllOrdersRequest {
    pub symbol: Option<String>,
}

#[async_trait::async_trait]
impl BinanceUsdmRequest for CancelAllOrdersRequest {
    fn endpoint(&self) -> &str {
        "/allOpenOrders"
    }

    fn method(&self) -> reqwest::Method {
        reqwest::Method::DELETE
    }

    fn parameters(&self) -> BTreeMap<String, serde_json::Value> {
        let mut params = BTreeMap::new();
        if let Some(ref symbol) = self.symbol {
            params.insert("symbol".to_string(), serde_json::Value::String(symbol.clone()));
        }
        params
    }
}
