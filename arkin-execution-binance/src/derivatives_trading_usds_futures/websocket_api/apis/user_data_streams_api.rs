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
pub trait UserDataStreamsApi: Send + Sync {
    async fn close_user_data_stream(
        &self,
        params: CloseUserDataStreamParams,
    ) -> anyhow::Result<WebsocketApiResponse<serde_json::Value>>;
    async fn keepalive_user_data_stream(
        &self,
        params: KeepaliveUserDataStreamParams,
    ) -> anyhow::Result<WebsocketApiResponse<Box<models::KeepaliveUserDataStreamResponseResult>>>;
    async fn start_user_data_stream(
        &self,
        params: StartUserDataStreamParams,
    ) -> anyhow::Result<WebsocketApiResponse<Box<models::StartUserDataStreamResponseResult>>>;
}

pub struct UserDataStreamsApiClient {
    websocket_api_base: Arc<WebsocketApi>,
}

impl UserDataStreamsApiClient {
    pub fn new(websocket_api_base: Arc<WebsocketApi>) -> Self {
        Self { websocket_api_base }
    }
}

/// Request parameters for the [`close_user_data_stream`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`close_user_data_stream`](#method.close_user_data_stream).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct CloseUserDataStreamParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`keepalive_user_data_stream`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`keepalive_user_data_stream`](#method.keepalive_user_data_stream).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct KeepaliveUserDataStreamParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

/// Request parameters for the [`start_user_data_stream`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`start_user_data_stream`](#method.start_user_data_stream).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct StartUserDataStreamParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
}

#[async_trait]
impl UserDataStreamsApi for UserDataStreamsApiClient {
    async fn close_user_data_stream(
        &self,
        params: CloseUserDataStreamParams,
    ) -> anyhow::Result<WebsocketApiResponse<serde_json::Value>> {
        let CloseUserDataStreamParams { id } = params;

        let mut payload: BTreeMap<String, Value> = BTreeMap::new();
        if let Some(value) = id {
            payload.insert("id".to_string(), serde_json::json!(value));
        }
        let payload = remove_empty_value(payload);

        self.websocket_api_base
            .send_message::<serde_json::Value>(
                "/userDataStream.stop".trim_start_matches('/'),
                payload,
                WebsocketMessageSendOptions {
                    is_signed: false,
                    with_api_key: true,
                },
            )
            .await
            .map_err(anyhow::Error::from)
    }

    async fn keepalive_user_data_stream(
        &self,
        params: KeepaliveUserDataStreamParams,
    ) -> anyhow::Result<WebsocketApiResponse<Box<models::KeepaliveUserDataStreamResponseResult>>> {
        let KeepaliveUserDataStreamParams { id } = params;

        let mut payload: BTreeMap<String, Value> = BTreeMap::new();
        if let Some(value) = id {
            payload.insert("id".to_string(), serde_json::json!(value));
        }
        let payload = remove_empty_value(payload);

        self.websocket_api_base
            .send_message::<Box<models::KeepaliveUserDataStreamResponseResult>>(
                "/userDataStream.ping".trim_start_matches('/'),
                payload,
                WebsocketMessageSendOptions {
                    is_signed: false,
                    with_api_key: true,
                },
            )
            .await
            .map_err(anyhow::Error::from)
    }

    async fn start_user_data_stream(
        &self,
        params: StartUserDataStreamParams,
    ) -> anyhow::Result<WebsocketApiResponse<Box<models::StartUserDataStreamResponseResult>>> {
        let StartUserDataStreamParams { id } = params;

        let mut payload: BTreeMap<String, Value> = BTreeMap::new();
        if let Some(value) = id {
            payload.insert("id".to_string(), serde_json::json!(value));
        }
        let payload = remove_empty_value(payload);

        self.websocket_api_base
            .send_message::<Box<models::StartUserDataStreamResponseResult>>(
                "/userDataStream.start".trim_start_matches('/'),
                payload,
                WebsocketMessageSendOptions {
                    is_signed: false,
                    with_api_key: true,
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
//     fn close_user_data_stream_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = UserDataStreamsApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = CloseUserDataStreamParams::builder().build().unwrap();
//                 client.close_user_data_stream(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv()).await.expect("send should occur").expect("channel closed");
//             let Message::Text(text) = sent else { panic!() };
//             let v: Value = serde_json::from_str(&text).unwrap();
//             let id = v["id"].as_str().unwrap();
//             assert_eq!(v["method"], "/userDataStream.stop".trim_start_matches('/'));

//             let mut resp_json: Value = serde_json::from_str(r#"{"id":"819e1b1b-8c06-485b-a13e-131326c69599","status":200,"result":{},"rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":2400,"count":2}]}"#).unwrap();
//             resp_json["id"] = id.into();

//             let raw_data = resp_json.get("result").or_else(|| resp_json.get("response")).expect("no response in JSON");
//             let expected_data: serde_json::Value = serde_json::from_value(raw_data.clone()).expect("should parse raw response");
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
//     fn close_user_data_stream_error_response() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = UserDataStreamsApiClient::new(ws_api.clone());

//             let handle = tokio::spawn(async move {
//                 let params = CloseUserDataStreamParams::builder().build().unwrap();
//                 client.close_user_data_stream(params).await
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
//     fn close_user_data_stream_request_timeout() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, _conn, mut rx) = setup().await;
//             let client = UserDataStreamsApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = CloseUserDataStreamParams::builder().build().unwrap();
//                 client.close_user_data_stream(params).await
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
//     fn keepalive_user_data_stream_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = UserDataStreamsApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = KeepaliveUserDataStreamParams::builder().build().unwrap();
//                 client.keepalive_user_data_stream(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv()).await.expect("send should occur").expect("channel closed");
//             let Message::Text(text) = sent else { panic!() };
//             let v: Value = serde_json::from_str(&text).unwrap();
//             let id = v["id"].as_str().unwrap();
//             assert_eq!(v["method"], "/userDataStream.ping".trim_start_matches('/'));

//             let mut resp_json: Value = serde_json::from_str(r#"{"id":"815d5fce-0880-4287-a567-80badf004c74","status":200,"result":{"listenKey":"3HBntNTepshgEdjIwSUIBgB9keLyOCg5qv3n6bYAtktG8ejcaW5HXz9Vx1JgIieg"},"rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":2400,"count":2}]}"#).unwrap();
//             resp_json["id"] = id.into();

//             let raw_data = resp_json.get("result").or_else(|| resp_json.get("response")).expect("no response in JSON");
//             let expected_data: Box<models::KeepaliveUserDataStreamResponseResult> = serde_json::from_value(raw_data.clone()).expect("should parse raw response");
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
//     fn keepalive_user_data_stream_error_response() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = UserDataStreamsApiClient::new(ws_api.clone());

//             let handle = tokio::spawn(async move {
//                 let params = KeepaliveUserDataStreamParams::builder().build().unwrap();
//                 client.keepalive_user_data_stream(params).await
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
//     fn keepalive_user_data_stream_request_timeout() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, _conn, mut rx) = setup().await;
//             let client = UserDataStreamsApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = KeepaliveUserDataStreamParams::builder().build().unwrap();
//                 client.keepalive_user_data_stream(params).await
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
//     fn start_user_data_stream_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = UserDataStreamsApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = StartUserDataStreamParams::builder().build().unwrap();
//                 client.start_user_data_stream(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv()).await.expect("send should occur").expect("channel closed");
//             let Message::Text(text) = sent else { panic!() };
//             let v: Value = serde_json::from_str(&text).unwrap();
//             let id = v["id"].as_str().unwrap();
//             assert_eq!(v["method"], "/userDataStream.start".trim_start_matches('/'));

//             let mut resp_json: Value = serde_json::from_str(r#"{"id":"d3df8a61-98ea-4fe0-8f4e-0fcea5d418b0","status":200,"result":{"listenKey":"xs0mRXdAKlIPDRFrlPcw0qI41Eh3ixNntmymGyhrhgqo7L6FuLaWArTD7RLP"},"rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":2400,"count":2}]}"#).unwrap();
//             resp_json["id"] = id.into();

//             let raw_data = resp_json.get("result").or_else(|| resp_json.get("response")).expect("no response in JSON");
//             let expected_data: Box<models::StartUserDataStreamResponseResult> = serde_json::from_value(raw_data.clone()).expect("should parse raw response");
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
//     fn start_user_data_stream_error_response() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = UserDataStreamsApiClient::new(ws_api.clone());

//             let handle = tokio::spawn(async move {
//                 let params = StartUserDataStreamParams::builder().build().unwrap();
//                 client.start_user_data_stream(params).await
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
//     fn start_user_data_stream_request_timeout() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, _conn, mut rx) = setup().await;
//             let client = UserDataStreamsApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = StartUserDataStreamParams::builder().build().unwrap();
//                 client.start_user_data_stream(params).await
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
