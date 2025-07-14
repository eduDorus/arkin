#![allow(unused_imports)]
use anyhow::Context;
use async_trait::async_trait;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Arc};
use typed_builder::TypedBuilder;

use crate::common::{
    models::{ParamBuildError, WebsocketApiResponse},
    utils::remove_empty_value,
    websocket::{WebsocketApi, WebsocketMessageSendOptions},
};
use crate::derivatives_trading_usds_futures::websocket_api::models;

#[async_trait]
pub trait MarketDataApi: Send + Sync {
    async fn order_book(
        &self,
        params: OrderBookParams,
    ) -> anyhow::Result<WebsocketApiResponse<Box<models::OrderBookResponseResult>>>;
    async fn symbol_order_book_ticker(
        &self,
        params: SymbolOrderBookTickerParams,
    ) -> anyhow::Result<WebsocketApiResponse<models::SymbolOrderBookTickerResponse>>;
    async fn symbol_price_ticker(
        &self,
        params: SymbolPriceTickerParams,
    ) -> anyhow::Result<WebsocketApiResponse<models::SymbolPriceTickerResponse>>;
}

pub struct MarketDataApiClient {
    websocket_api_base: Arc<WebsocketApi>,
}

impl MarketDataApiClient {
    pub fn new(websocket_api_base: Arc<WebsocketApi>) -> Self {
        Self { websocket_api_base }
    }
}

/// Request parameters for the [`order_book`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`order_book`](#method.order_book).
#[derive(Clone, Debug, TypedBuilder)]
pub struct OrderBookParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
    /// Default 500; Valid limits:[5, 10, 20, 50, 100, 500, 1000]
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
}

/// Request parameters for the [`symbol_order_book_ticker`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`symbol_order_book_ticker`](#method.symbol_order_book_ticker).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct SymbolOrderBookTickerParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
}

/// Request parameters for the [`symbol_price_ticker`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`symbol_price_ticker`](#method.symbol_price_ticker).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct SymbolPriceTickerParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
}

#[async_trait]
impl MarketDataApi for MarketDataApiClient {
    async fn order_book(
        &self,
        params: OrderBookParams,
    ) -> anyhow::Result<WebsocketApiResponse<Box<models::OrderBookResponseResult>>> {
        let OrderBookParams { symbol, id, limit } = params;

        let mut payload: BTreeMap<String, Value> = BTreeMap::new();
        payload.insert("symbol".to_string(), serde_json::json!(symbol));
        if let Some(value) = id {
            payload.insert("id".to_string(), serde_json::json!(value));
        }
        if let Some(value) = limit {
            payload.insert("limit".to_string(), serde_json::json!(value));
        }
        let payload = remove_empty_value(payload);

        self.websocket_api_base
            .send_message::<Box<models::OrderBookResponseResult>>(
                "/depth".trim_start_matches('/'),
                payload,
                WebsocketMessageSendOptions {
                    is_signed: false,
                    with_api_key: false,
                },
            )
            .await
            .map_err(anyhow::Error::from)
    }

    async fn symbol_order_book_ticker(
        &self,
        params: SymbolOrderBookTickerParams,
    ) -> anyhow::Result<WebsocketApiResponse<models::SymbolOrderBookTickerResponse>> {
        let SymbolOrderBookTickerParams { id, symbol } = params;

        let mut payload: BTreeMap<String, Value> = BTreeMap::new();
        if let Some(value) = id {
            payload.insert("id".to_string(), serde_json::json!(value));
        }
        if let Some(value) = symbol {
            payload.insert("symbol".to_string(), serde_json::json!(value));
        }
        let payload = remove_empty_value(payload);

        self.websocket_api_base
            .send_message::<models::SymbolOrderBookTickerResponse>(
                "/ticker.book".trim_start_matches('/'),
                payload,
                WebsocketMessageSendOptions {
                    is_signed: false,
                    with_api_key: false,
                },
            )
            .await
            .map_err(anyhow::Error::from)
    }

    async fn symbol_price_ticker(
        &self,
        params: SymbolPriceTickerParams,
    ) -> anyhow::Result<WebsocketApiResponse<models::SymbolPriceTickerResponse>> {
        let SymbolPriceTickerParams { id, symbol } = params;

        let mut payload: BTreeMap<String, Value> = BTreeMap::new();
        if let Some(value) = id {
            payload.insert("id".to_string(), serde_json::json!(value));
        }
        if let Some(value) = symbol {
            payload.insert("symbol".to_string(), serde_json::json!(value));
        }
        let payload = remove_empty_value(payload);

        self.websocket_api_base
            .send_message::<models::SymbolPriceTickerResponse>(
                "/ticker.price".trim_start_matches('/'),
                payload,
                WebsocketMessageSendOptions {
                    is_signed: false,
                    with_api_key: false,
                },
            )
            .await
            .map_err(anyhow::Error::from)
    }
}

// #[cfg(all(test, feature = "derivatives_trading_usds_futures"))]
// mod tests {
//     use super::*;
//     use crate::common::websocket::{WebsocketApi, WebsocketConnection, WebsocketHandler};
//     use crate::config::ConfigurationWebsocketApi;
//     use crate::errors::WebsocketError;
//     use crate::models::WebsocketApiRateLimit;
//     use crate::TOKIO_SHARED_RT;
//     use serde_json::{json, Value};
//     use tokio::spawn;
//     use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};
//     use tokio::time::{timeout, Duration};
//     use tokio_tungstenite::tungstenite::Message;

//     async fn setup() -> (Arc<WebsocketApi>, Arc<WebsocketConnection>, UnboundedReceiver<Message>) {
//         let conn = WebsocketConnection::new("test-conn");
//         let (tx, rx) = unbounded_channel::<Message>();
//         {
//             let mut conn_state = conn.state.lock().await;
//             conn_state.ws_write_tx = Some(tx);
//         }

//         let config = ConfigurationWebsocketApi::builder()
//             .api_key("key")
//             .api_secret("secret")
//             .build()
//             .expect("Failed to build configuration");
//         let ws_api = WebsocketApi::new(config, vec![conn.clone()]);
//         conn.set_handler(ws_api.clone() as Arc<dyn WebsocketHandler>).await;
//         ws_api.clone().connect().await.unwrap();

//         (ws_api, conn, rx)
//     }

//     #[test]
//     fn order_book_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = MarketDataApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = OrderBookParams::builder("symbol_example".to_string(),).build().unwrap();
//                 client.order_book(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv()).await.expect("send should occur").expect("channel closed");
//             let Message::Text(text) = sent else { panic!() };
//             let v: Value = serde_json::from_str(&text).unwrap();
//             let id = v["id"].as_str().unwrap();
//             assert_eq!(v["method"], "/depth".trim_start_matches('/'));

//             let mut resp_json: Value = serde_json::from_str(r#"{"id":"51e2affb-0aba-4821-ba75-f2625006eb43","status":200,"result":{"lastUpdateId":1027024,"E":1589436922972,"T":1589436922959,"bids":[["4.00000000","431.00000000"]],"asks":[["4.00000200","12.00000000"]]},"rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":2400,"count":5}]}"#).unwrap();
//             resp_json["id"] = id.into();

//             let raw_data = resp_json.get("result").or_else(|| resp_json.get("response")).expect("no response in JSON");
//             let expected_data: Box<models::OrderBookResponseResult> = serde_json::from_value(raw_data.clone()).expect("should parse raw response");
//             let empty_array = Value::Array(vec![]);
//             let raw_rate_limits = resp_json.get("rateLimits").unwrap_or(&empty_array);
//             let expected_rate_limits: Option<Vec<WebsocketApiRateLimit>> =
//                 match raw_rate_limits.as_array() {
//                     Some(arr) if arr.is_empty() => None,
//                     Some(_) => Some(serde_json::from_value(raw_rate_limits.clone()).expect("should parse rateLimits array")),
//                     None => None,
//                 };

//             WebsocketHandler::on_message(&*ws_api, resp_json.to_string(), conn.clone()).await;

//             let response = timeout(Duration::from_secs(1), handle).await.expect("task done").expect("no panic").expect("no error");

//             let response_rate_limits = response.rate_limits.clone();
//             let response_data = response.data().expect("deserialize data");

//             assert_eq!(response_rate_limits, expected_rate_limits);
//             assert_eq!(response_data, expected_data);
//         });
//     }

//     #[test]
//     fn order_book_error_response() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = MarketDataApiClient::new(ws_api.clone());

//             let handle = tokio::spawn(async move {
//                 let params = OrderBookParams::builder("symbol_example".to_string(),).build().unwrap();
//                 client.order_book(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
//             let Message::Text(text) = sent else { panic!() };
//             let v: Value = serde_json::from_str(&text).unwrap();
//             let id = v["id"].as_str().unwrap().to_string();

//             let resp_json = json!({
//                 "id": id,
//                 "status": 400,
//                     "error": {
//                         "code": -2010,
//                         "msg": "Account has insufficient balance for requested action.",
//                     },
//                     "rateLimits": [
//                         {
//                             "rateLimitType": "ORDERS",
//                             "interval": "SECOND",
//                             "intervalNum": 10,
//                             "limit": 50,
//                             "count": 13
//                         },
//                     ],
//             });
//             WebsocketHandler::on_message(&*ws_api, resp_json.to_string(), conn.clone()).await;

//             let join = timeout(Duration::from_secs(1), handle).await.unwrap();
//             match join {
//                 Ok(Err(e)) => {
//                     let msg = e.to_string();
//                     assert!(
//                         msg.contains("Server‐side response error (code -2010): Account has insufficient balance for requested action."),
//                         "Expected error msg to contain server error, got: {msg}"
//                     );
//                 }
//                 Ok(Ok(_)) => panic!("Expected error"),
//                 Err(_) => panic!("Task panicked"),
//             }
//         });
//     }

//     #[test]
//     fn order_book_request_timeout() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, _conn, mut rx) = setup().await;
//             let client = MarketDataApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = OrderBookParams::builder("symbol_example".to_string()).build().unwrap();
//                 client.order_book(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv())
//                 .await
//                 .expect("send should occur")
//                 .expect("channel closed");
//             let Message::Text(text) = sent else {
//                 panic!("expected Message Text")
//             };

//             let _: Value = serde_json::from_str(&text).unwrap();

//             let result = handle.await.expect("task completed");
//             match result {
//                 Err(e) => {
//                     if let Some(inner) = e.downcast_ref::<WebsocketError>() {
//                         assert!(matches!(inner, WebsocketError::Timeout));
//                     } else {
//                         panic!("Unexpected error type: {:?}", e);
//                     }
//                 }
//                 Ok(_) => panic!("Expected timeout error"),
//             }
//         });
//     }

//     #[test]
//     fn symbol_order_book_ticker_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = MarketDataApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = SymbolOrderBookTickerParams::builder().build().unwrap();
//                 client.symbol_order_book_ticker(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv()).await.expect("send should occur").expect("channel closed");
//             let Message::Text(text) = sent else { panic!() };
//             let v: Value = serde_json::from_str(&text).unwrap();
//             let id = v["id"].as_str().unwrap();
//             assert_eq!(v["method"], "/ticker.book".trim_start_matches('/'));

//             let mut resp_json: Value = serde_json::from_str(r#"{"id":"9d32157c-a556-4d27-9866-66760a174b57","status":200,"result":{"lastUpdateId":1027024,"symbol":"BTCUSDT","bidPrice":"4.00000000","bidQty":"431.00000000","askPrice":"4.00000200","askQty":"9.00000000","time":1589437530011},"rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":2400,"count":2}]}"#).unwrap();
//             resp_json["id"] = id.into();

//             let raw_data = resp_json.get("result").or_else(|| resp_json.get("response")).expect("no response in JSON");
//             let expected_data: models::SymbolOrderBookTickerResponse = serde_json::from_value(raw_data.clone()).expect("should parse raw response");
//             let empty_array = Value::Array(vec![]);
//             let raw_rate_limits = resp_json.get("rateLimits").unwrap_or(&empty_array);
//             let expected_rate_limits: Option<Vec<WebsocketApiRateLimit>> =
//                 match raw_rate_limits.as_array() {
//                     Some(arr) if arr.is_empty() => None,
//                     Some(_) => Some(serde_json::from_value(raw_rate_limits.clone()).expect("should parse rateLimits array")),
//                     None => None,
//                 };

//             WebsocketHandler::on_message(&*ws_api, resp_json.to_string(), conn.clone()).await;

//             let response = timeout(Duration::from_secs(1), handle).await.expect("task done").expect("no panic").expect("no error");

//             let response_rate_limits = response.rate_limits.clone();
//             let response_data = response.data().expect("deserialize data");

//             assert_eq!(response_rate_limits, expected_rate_limits);
//             assert_eq!(response_data, expected_data);
//         });
//     }

//     #[test]
//     fn symbol_order_book_ticker_error_response() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = MarketDataApiClient::new(ws_api.clone());

//             let handle = tokio::spawn(async move {
//                 let params = SymbolOrderBookTickerParams::builder().build().unwrap();
//                 client.symbol_order_book_ticker(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
//             let Message::Text(text) = sent else { panic!() };
//             let v: Value = serde_json::from_str(&text).unwrap();
//             let id = v["id"].as_str().unwrap().to_string();

//             let resp_json = json!({
//                 "id": id,
//                 "status": 400,
//                     "error": {
//                         "code": -2010,
//                         "msg": "Account has insufficient balance for requested action.",
//                     },
//                     "rateLimits": [
//                         {
//                             "rateLimitType": "ORDERS",
//                             "interval": "SECOND",
//                             "intervalNum": 10,
//                             "limit": 50,
//                             "count": 13
//                         },
//                     ],
//             });
//             WebsocketHandler::on_message(&*ws_api, resp_json.to_string(), conn.clone()).await;

//             let join = timeout(Duration::from_secs(1), handle).await.unwrap();
//             match join {
//                 Ok(Err(e)) => {
//                     let msg = e.to_string();
//                     assert!(
//                         msg.contains("Server‐side response error (code -2010): Account has insufficient balance for requested action."),
//                         "Expected error msg to contain server error, got: {msg}"
//                     );
//                 }
//                 Ok(Ok(_)) => panic!("Expected error"),
//                 Err(_) => panic!("Task panicked"),
//             }
//         });
//     }

//     #[test]
//     fn symbol_order_book_ticker_request_timeout() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, _conn, mut rx) = setup().await;
//             let client = MarketDataApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = SymbolOrderBookTickerParams::builder().build().unwrap();
//                 client.symbol_order_book_ticker(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv())
//                 .await
//                 .expect("send should occur")
//                 .expect("channel closed");
//             let Message::Text(text) = sent else {
//                 panic!("expected Message Text")
//             };

//             let _: Value = serde_json::from_str(&text).unwrap();

//             let result = handle.await.expect("task completed");
//             match result {
//                 Err(e) => {
//                     if let Some(inner) = e.downcast_ref::<WebsocketError>() {
//                         assert!(matches!(inner, WebsocketError::Timeout));
//                     } else {
//                         panic!("Unexpected error type: {:?}", e);
//                     }
//                 }
//                 Ok(_) => panic!("Expected timeout error"),
//             }
//         });
//     }

//     #[test]
//     fn symbol_price_ticker_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = MarketDataApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = SymbolPriceTickerParams::builder().build().unwrap();
//                 client.symbol_price_ticker(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv()).await.expect("send should occur").expect("channel closed");
//             let Message::Text(text) = sent else { panic!() };
//             let v: Value = serde_json::from_str(&text).unwrap();
//             let id = v["id"].as_str().unwrap();
//             assert_eq!(v["method"], "/ticker.price".trim_start_matches('/'));

//             let mut resp_json: Value = serde_json::from_str(r#"{"id":"9d32157c-a556-4d27-9866-66760a174b57","status":200,"result":{"symbol":"BTCUSDT","price":"6000.01","time":1589437530011},"rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":2400,"count":2}]}"#).unwrap();
//             resp_json["id"] = id.into();

//             let raw_data = resp_json.get("result").or_else(|| resp_json.get("response")).expect("no response in JSON");
//             let expected_data: models::SymbolPriceTickerResponse = serde_json::from_value(raw_data.clone()).expect("should parse raw response");
//             let empty_array = Value::Array(vec![]);
//             let raw_rate_limits = resp_json.get("rateLimits").unwrap_or(&empty_array);
//             let expected_rate_limits: Option<Vec<WebsocketApiRateLimit>> =
//                 match raw_rate_limits.as_array() {
//                     Some(arr) if arr.is_empty() => None,
//                     Some(_) => Some(serde_json::from_value(raw_rate_limits.clone()).expect("should parse rateLimits array")),
//                     None => None,
//                 };

//             WebsocketHandler::on_message(&*ws_api, resp_json.to_string(), conn.clone()).await;

//             let response = timeout(Duration::from_secs(1), handle).await.expect("task done").expect("no panic").expect("no error");

//             let response_rate_limits = response.rate_limits.clone();
//             let response_data = response.data().expect("deserialize data");

//             assert_eq!(response_rate_limits, expected_rate_limits);
//             assert_eq!(response_data, expected_data);
//         });
//     }

//     #[test]
//     fn symbol_price_ticker_error_response() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = MarketDataApiClient::new(ws_api.clone());

//             let handle = tokio::spawn(async move {
//                 let params = SymbolPriceTickerParams::builder().build().unwrap();
//                 client.symbol_price_ticker(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv()).await.unwrap().unwrap();
//             let Message::Text(text) = sent else { panic!() };
//             let v: Value = serde_json::from_str(&text).unwrap();
//             let id = v["id"].as_str().unwrap().to_string();

//             let resp_json = json!({
//                 "id": id,
//                 "status": 400,
//                     "error": {
//                         "code": -2010,
//                         "msg": "Account has insufficient balance for requested action.",
//                     },
//                     "rateLimits": [
//                         {
//                             "rateLimitType": "ORDERS",
//                             "interval": "SECOND",
//                             "intervalNum": 10,
//                             "limit": 50,
//                             "count": 13
//                         },
//                     ],
//             });
//             WebsocketHandler::on_message(&*ws_api, resp_json.to_string(), conn.clone()).await;

//             let join = timeout(Duration::from_secs(1), handle).await.unwrap();
//             match join {
//                 Ok(Err(e)) => {
//                     let msg = e.to_string();
//                     assert!(
//                         msg.contains("Server‐side response error (code -2010): Account has insufficient balance for requested action."),
//                         "Expected error msg to contain server error, got: {msg}"
//                     );
//                 }
//                 Ok(Ok(_)) => panic!("Expected error"),
//                 Err(_) => panic!("Task panicked"),
//             }
//         });
//     }

//     #[test]
//     fn symbol_price_ticker_request_timeout() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, _conn, mut rx) = setup().await;
//             let client = MarketDataApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = SymbolPriceTickerParams::builder().build().unwrap();
//                 client.symbol_price_ticker(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv())
//                 .await
//                 .expect("send should occur")
//                 .expect("channel closed");
//             let Message::Text(text) = sent else {
//                 panic!("expected Message Text")
//             };

//             let _: Value = serde_json::from_str(&text).unwrap();

//             let result = handle.await.expect("task completed");
//             match result {
//                 Err(e) => {
//                     if let Some(inner) = e.downcast_ref::<WebsocketError>() {
//                         assert!(matches!(inner, WebsocketError::Timeout));
//                     } else {
//                         panic!("Unexpected error type: {:?}", e);
//                     }
//                 }
//                 Ok(_) => panic!("Expected timeout error"),
//             }
//         });
//     }
// }
