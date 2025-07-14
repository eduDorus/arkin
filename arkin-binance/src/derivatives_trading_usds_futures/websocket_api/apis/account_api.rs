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
pub trait AccountApi: Send + Sync {
    async fn account_information(
        &self,
        params: AccountInformationParams,
    ) -> anyhow::Result<WebsocketApiResponse<Box<models::AccountInformationResponseResult>>>;
    async fn account_information_v2(
        &self,
        params: AccountInformationV2Params,
    ) -> anyhow::Result<WebsocketApiResponse<Box<models::AccountInformationV2ResponseResult>>>;
    async fn futures_account_balance(
        &self,
        params: FuturesAccountBalanceParams,
    ) -> anyhow::Result<WebsocketApiResponse<Vec<models::FuturesAccountBalanceV2ResponseResultInner>>>;
    async fn futures_account_balance_v2(
        &self,
        params: FuturesAccountBalanceV2Params,
    ) -> anyhow::Result<WebsocketApiResponse<Vec<models::FuturesAccountBalanceV2ResponseResultInner>>>;
}

pub struct AccountApiClient {
    websocket_api_base: Arc<WebsocketApi>,
}

impl AccountApiClient {
    pub fn new(websocket_api_base: Arc<WebsocketApi>) -> Self {
        Self { websocket_api_base }
    }
}

/// Request parameters for the [`account_information`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`account_information`](#method.account_information).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct AccountInformationParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`account_information_v2`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`account_information_v2`](#method.account_information_v2).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct AccountInformationV2Params {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`futures_account_balance`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`futures_account_balance`](#method.futures_account_balance).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct FuturesAccountBalanceParams {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`futures_account_balance_v2`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`futures_account_balance_v2`](#method.futures_account_balance_v2).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct FuturesAccountBalanceV2Params {
    /// Unique WebSocket request ID.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub id: Option<String>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

#[async_trait]
impl AccountApi for AccountApiClient {
    async fn account_information(
        &self,
        params: AccountInformationParams,
    ) -> anyhow::Result<WebsocketApiResponse<Box<models::AccountInformationResponseResult>>> {
        let AccountInformationParams { id, recv_window } = params;

        let mut payload: BTreeMap<String, Value> = BTreeMap::new();
        if let Some(value) = id {
            payload.insert("id".to_string(), serde_json::json!(value));
        }
        if let Some(value) = recv_window {
            payload.insert("recvWindow".to_string(), serde_json::json!(value));
        }
        let payload = remove_empty_value(payload);

        self.websocket_api_base
            .send_message::<Box<models::AccountInformationResponseResult>>(
                "/account.status".trim_start_matches('/'),
                payload,
                WebsocketMessageSendOptions {
                    is_signed: true,
                    with_api_key: false,
                },
            )
            .await
            .map_err(anyhow::Error::from)
    }

    async fn account_information_v2(
        &self,
        params: AccountInformationV2Params,
    ) -> anyhow::Result<WebsocketApiResponse<Box<models::AccountInformationV2ResponseResult>>> {
        let AccountInformationV2Params { id, recv_window } = params;

        let mut payload: BTreeMap<String, Value> = BTreeMap::new();
        if let Some(value) = id {
            payload.insert("id".to_string(), serde_json::json!(value));
        }
        if let Some(value) = recv_window {
            payload.insert("recvWindow".to_string(), serde_json::json!(value));
        }
        let payload = remove_empty_value(payload);

        self.websocket_api_base
            .send_message::<Box<models::AccountInformationV2ResponseResult>>(
                "/v2/account.status".trim_start_matches('/'),
                payload,
                WebsocketMessageSendOptions {
                    is_signed: true,
                    with_api_key: false,
                },
            )
            .await
            .map_err(anyhow::Error::from)
    }

    async fn futures_account_balance(
        &self,
        params: FuturesAccountBalanceParams,
    ) -> anyhow::Result<WebsocketApiResponse<Vec<models::FuturesAccountBalanceV2ResponseResultInner>>> {
        let FuturesAccountBalanceParams { id, recv_window } = params;

        let mut payload: BTreeMap<String, Value> = BTreeMap::new();
        if let Some(value) = id {
            payload.insert("id".to_string(), serde_json::json!(value));
        }
        if let Some(value) = recv_window {
            payload.insert("recvWindow".to_string(), serde_json::json!(value));
        }
        let payload = remove_empty_value(payload);

        self.websocket_api_base
            .send_message::<Vec<models::FuturesAccountBalanceV2ResponseResultInner>>(
                "/account.balance".trim_start_matches('/'),
                payload,
                WebsocketMessageSendOptions {
                    is_signed: true,
                    with_api_key: false,
                },
            )
            .await
            .map_err(anyhow::Error::from)
    }

    async fn futures_account_balance_v2(
        &self,
        params: FuturesAccountBalanceV2Params,
    ) -> anyhow::Result<WebsocketApiResponse<Vec<models::FuturesAccountBalanceV2ResponseResultInner>>> {
        let FuturesAccountBalanceV2Params { id, recv_window } = params;

        let mut payload: BTreeMap<String, Value> = BTreeMap::new();
        if let Some(value) = id {
            payload.insert("id".to_string(), serde_json::json!(value));
        }
        if let Some(value) = recv_window {
            payload.insert("recvWindow".to_string(), serde_json::json!(value));
        }
        let payload = remove_empty_value(payload);

        self.websocket_api_base
            .send_message::<Vec<models::FuturesAccountBalanceV2ResponseResultInner>>(
                "/v2/account.balance".trim_start_matches('/'),
                payload,
                WebsocketMessageSendOptions {
                    is_signed: true,
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
//     fn account_information_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = AccountApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = AccountInformationParams::builder().build().unwrap();
//                 client.account_information(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv()).await.expect("send should occur").expect("channel closed");
//             let Message::Text(text) = sent else { panic!() };
//             let v: Value = serde_json::from_str(&text).unwrap();
//             let id = v["id"].as_str().unwrap();
//             assert_eq!(v["method"], "/account.status".trim_start_matches('/'));

//             let mut resp_json: Value = serde_json::from_str(r#"{"id":"605a6d20-6588-4cb9-afa0-b0ab087507ba","status":200,"result":{"feeTier":0,"canTrade":true,"canDeposit":true,"canWithdraw":true,"updateTime":0,"multiAssetsMargin":true,"tradeGroupId":-1,"totalInitialMargin":"0.00000000","totalMaintMargin":"0.00000000","totalWalletBalance":"126.72469206","totalUnrealizedProfit":"0.00000000","totalMarginBalance":"126.72469206","totalPositionInitialMargin":"0.00000000","totalOpenOrderInitialMargin":"0.00000000","totalCrossWalletBalance":"126.72469206","totalCrossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"126.72469206","assets":[{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1625474304765},{"asset":"BUSD","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"103.12345678","maxWithdrawAmount":"103.12345678","marginAvailable":true,"updateTime":1625474304765},{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1625474304765},{"asset":"BUSD","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"103.12345678","marginAvailable":true,"updateTime":1625474304765}],"positions":[{"symbol":"BTCUSDT","initialMargin":"0","maintMargin":"0","unrealizedProfit":"0.00000000","positionInitialMargin":"0","openOrderInitialMargin":"0","leverage":"100","isolated":true,"entryPrice":"0.00000","maxNotional":"250000","bidNotional":"0","askNotional":"0","positionSide":"BOTH","positionAmt":"0","updateTime":0},{"symbol":"BTCUSDT","initialMargin":"0","maintMargin":"0","unrealizedProfit":"0.00000000","positionInitialMargin":"0","openOrderInitialMargin":"0","leverage":"100","isolated":true,"entryPrice":"0.00000","breakEvenPrice":"0.0","maxNotional":"250000","bidNotional":"0","askNotional":"0","positionSide":"BOTH","positionAmt":"0","updateTime":0}]},"rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":2400,"count":20}]}"#).unwrap();
//             resp_json["id"] = id.into();

//             let raw_data = resp_json.get("result").or_else(|| resp_json.get("response")).expect("no response in JSON");
//             let expected_data: Box<models::AccountInformationResponseResult> = serde_json::from_value(raw_data.clone()).expect("should parse raw response");
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
//     fn account_information_error_response() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = AccountApiClient::new(ws_api.clone());

//             let handle = tokio::spawn(async move {
//                 let params = AccountInformationParams::builder().build().unwrap();
//                 client.account_information(params).await
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
//     fn account_information_request_timeout() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, _conn, mut rx) = setup().await;
//             let client = AccountApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = AccountInformationParams::builder().build().unwrap();
//                 client.account_information(params).await
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
//     fn account_information_v2_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = AccountApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = AccountInformationV2Params::builder().build().unwrap();
//                 client.account_information_v2(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv()).await.expect("send should occur").expect("channel closed");
//             let Message::Text(text) = sent else { panic!() };
//             let v: Value = serde_json::from_str(&text).unwrap();
//             let id = v["id"].as_str().unwrap();
//             assert_eq!(v["method"], "/v2/account.status".trim_start_matches('/'));

//             let mut resp_json: Value = serde_json::from_str(r#"{"id":"605a6d20-6588-4cb9-afa0-b0ab087507ba","status":200,"result":{"totalInitialMargin":"0.00000000","totalMaintMargin":"0.00000000","totalWalletBalance":"126.72469206","totalUnrealizedProfit":"0.00000000","totalMarginBalance":"126.72469206","totalPositionInitialMargin":"0.00000000","totalOpenOrderInitialMargin":"0.00000000","totalCrossWalletBalance":"126.72469206","totalCrossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"126.72469206","assets":[{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","updateTime":1625474304765},{"asset":"USDC","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"103.12345678","updateTime":1625474304765},{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1625474304765},{"asset":"BUSD","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"103.12345678","marginAvailable":true,"updateTime":1625474304765}],"positions":[{"symbol":"BTCUSDT","positionSide":"BOTH","positionAmt":"1.000","unrealizedProfit":"0.00000000","isolatedMargin":"0.00000000","notional":"0","isolatedWallet":"0","initialMargin":"0","maintMargin":"0","updateTime":0},{"symbol":"BTCUSDT","positionSide":"BOTH","positionAmt":"1.000","unrealizedProfit":"0.00000000","isolatedMargin":"0.00000000","notional":"0","isolatedWallet":"0","initialMargin":"0","maintMargin":"0","updateTime":0}]},"rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":2400,"count":20}]}"#).unwrap();
//             resp_json["id"] = id.into();

//             let raw_data = resp_json.get("result").or_else(|| resp_json.get("response")).expect("no response in JSON");
//             let expected_data: Box<models::AccountInformationV2ResponseResult> = serde_json::from_value(raw_data.clone()).expect("should parse raw response");
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
//     fn account_information_v2_error_response() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = AccountApiClient::new(ws_api.clone());

//             let handle = tokio::spawn(async move {
//                 let params = AccountInformationV2Params::builder().build().unwrap();
//                 client.account_information_v2(params).await
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
//     fn account_information_v2_request_timeout() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, _conn, mut rx) = setup().await;
//             let client = AccountApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = AccountInformationV2Params::builder().build().unwrap();
//                 client.account_information_v2(params).await
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
//     fn futures_account_balance_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = AccountApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = FuturesAccountBalanceParams::builder().build().unwrap();
//                 client.futures_account_balance(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv()).await.expect("send should occur").expect("channel closed");
//             let Message::Text(text) = sent else { panic!() };
//             let v: Value = serde_json::from_str(&text).unwrap();
//             let id = v["id"].as_str().unwrap();
//             assert_eq!(v["method"], "/account.balance".trim_start_matches('/'));

//             let mut resp_json: Value = serde_json::from_str(r#"{"id":"605a6d20-6588-4cb9-afa0-b0ab087507ba","status":200,"result":[{"accountAlias":"SgsR","asset":"USDT","balance":"122607.35137903","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1617939110373}],"rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":2400,"count":20}]}"#).unwrap();
//             resp_json["id"] = id.into();

//             let raw_data = resp_json.get("result").or_else(|| resp_json.get("response")).expect("no response in JSON");
//             let expected_data: Vec<models::FuturesAccountBalanceV2ResponseResultInner> = serde_json::from_value(raw_data.clone()).expect("should parse raw response");
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
//     fn futures_account_balance_error_response() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = AccountApiClient::new(ws_api.clone());

//             let handle = tokio::spawn(async move {
//                 let params = FuturesAccountBalanceParams::builder().build().unwrap();
//                 client.futures_account_balance(params).await
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
//     fn futures_account_balance_request_timeout() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, _conn, mut rx) = setup().await;
//             let client = AccountApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = FuturesAccountBalanceParams::builder().build().unwrap();
//                 client.futures_account_balance(params).await
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
//     fn futures_account_balance_v2_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = AccountApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = FuturesAccountBalanceV2Params::builder().build().unwrap();
//                 client.futures_account_balance_v2(params).await
//             });

//             let sent = timeout(Duration::from_secs(1), rx.recv()).await.expect("send should occur").expect("channel closed");
//             let Message::Text(text) = sent else { panic!() };
//             let v: Value = serde_json::from_str(&text).unwrap();
//             let id = v["id"].as_str().unwrap();
//             assert_eq!(v["method"], "/v2/account.balance".trim_start_matches('/'));

//             let mut resp_json: Value = serde_json::from_str(r#"{"id":"605a6d20-6588-4cb9-afa0-b0ab087507ba","status":200,"result":[{"accountAlias":"SgsR","asset":"USDT","balance":"122607.35137903","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1617939110373}],"rateLimits":[{"rateLimitType":"REQUEST_WEIGHT","interval":"MINUTE","intervalNum":1,"limit":2400,"count":20}]}"#).unwrap();
//             resp_json["id"] = id.into();

//             let raw_data = resp_json.get("result").or_else(|| resp_json.get("response")).expect("no response in JSON");
//             let expected_data: Vec<models::FuturesAccountBalanceV2ResponseResultInner> = serde_json::from_value(raw_data.clone()).expect("should parse raw response");
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
//     fn futures_account_balance_v2_error_response() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, conn, mut rx) = setup().await;
//             let client = AccountApiClient::new(ws_api.clone());

//             let handle = tokio::spawn(async move {
//                 let params = FuturesAccountBalanceV2Params::builder().build().unwrap();
//                 client.futures_account_balance_v2(params).await
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
//     fn futures_account_balance_v2_request_timeout() {
//         TOKIO_SHARED_RT.block_on(async {
//             let (ws_api, _conn, mut rx) = setup().await;
//             let client = AccountApiClient::new(ws_api.clone());

//             let handle = spawn(async move {
//                 let params = FuturesAccountBalanceV2Params::builder().build().unwrap();
//                 client.futures_account_balance_v2(params).await
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
