
#![allow(unused_imports)]
use async_trait::async_trait;
use reqwest;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;

use crate::common::{
    config::ConfigurationRestApi,
    models::{ParamBuildError, RestApiResponse},
    utils::send_request,
};
use crate::derivatives_trading_usds_futures::rest_api::models;

const HAS_TIME_UNIT: bool = false;

#[async_trait]
pub trait UserDataStreamsApi: Send + Sync {
    async fn close_user_data_stream(&self) -> anyhow::Result<RestApiResponse<Value>>;
    async fn keepalive_user_data_stream(
        &self,
    ) -> anyhow::Result<RestApiResponse<models::KeepaliveUserDataStreamResponse>>;
    async fn start_user_data_stream(&self) -> anyhow::Result<RestApiResponse<models::StartUserDataStreamResponse>>;
}

#[derive(Debug, Clone)]
pub struct UserDataStreamsApiClient {
    configuration: ConfigurationRestApi,
}

impl UserDataStreamsApiClient {
    pub fn new(configuration: ConfigurationRestApi) -> Self {
        Self { configuration }
    }
}

#[async_trait]
impl UserDataStreamsApi for UserDataStreamsApiClient {
    async fn close_user_data_stream(&self) -> anyhow::Result<RestApiResponse<Value>> {
        let query_params = BTreeMap::new();

        send_request::<Value>(
            &self.configuration,
            "/fapi/v1/listenKey",
            reqwest::Method::DELETE,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn keepalive_user_data_stream(
        &self,
    ) -> anyhow::Result<RestApiResponse<models::KeepaliveUserDataStreamResponse>> {
        let query_params = BTreeMap::new();

        send_request::<models::KeepaliveUserDataStreamResponse>(
            &self.configuration,
            "/fapi/v1/listenKey",
            reqwest::Method::PUT,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }

    async fn start_user_data_stream(&self) -> anyhow::Result<RestApiResponse<models::StartUserDataStreamResponse>> {
        let query_params = BTreeMap::new();

        send_request::<models::StartUserDataStreamResponse>(
            &self.configuration,
            "/fapi/v1/listenKey",
            reqwest::Method::POST,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            false,
        )
        .await
    }
}

// #[cfg(all(test, feature = "derivatives_trading_usds_futures"))]
// mod tests {
//     use super::*;
//     use crate::TOKIO_SHARED_RT;
//     use crate::{errors::ConnectorError, models::DataFuture, models::RestApiRateLimit};
//     use async_trait::async_trait;
//     use std::collections::HashMap;

//     struct DummyRestApiResponse<T> {
//         inner: Box<dyn FnOnce() -> DataFuture<Result<T, ConnectorError>> + Send + Sync>,
//         status: u16,
//         headers: HashMap<String, String>,
//         rate_limits: Option<Vec<RestApiRateLimit>>,
//     }

//     impl<T> From<DummyRestApiResponse<T>> for RestApiResponse<T> {
//         fn from(dummy: DummyRestApiResponse<T>) -> Self {
//             Self {
//                 data_fn: dummy.inner,
//                 status: dummy.status,
//                 headers: dummy.headers,
//                 rate_limits: dummy.rate_limits,
//             }
//         }
//     }

//     struct MockUserDataStreamsApiClient {
//         force_error: bool,
//     }

//     #[async_trait]
//     impl UserDataStreamsApi for MockUserDataStreamsApiClient {
//         async fn close_user_data_stream(&self) -> anyhow::Result<RestApiResponse<Value>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let dummy_response = Value::Null;

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn keepalive_user_data_stream(
//             &self,
//         ) -> anyhow::Result<RestApiResponse<models::KeepaliveUserDataStreamResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"listenKey":"3HBntNTepshgEdjIwSUIBgB9keLyOCg5qv3n6bYAtktG8ejcaW5HXz9Vx1JgIieg"}"#,
//             )
//             .unwrap();
//             let dummy_response: models::KeepaliveUserDataStreamResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::KeepaliveUserDataStreamResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn start_user_data_stream(&self) -> anyhow::Result<RestApiResponse<models::StartUserDataStreamResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"listenKey":"pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1"}"#,
//             )
//             .unwrap();
//             let dummy_response: models::StartUserDataStreamResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::StartUserDataStreamResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }
//     }

//     #[test]
//     fn close_user_data_stream_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockUserDataStreamsApiClient { force_error: false };

//             let expected_response = Value::Null;

//             let resp = client.close_user_data_stream().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn close_user_data_stream_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockUserDataStreamsApiClient { force_error: false };

//             let expected_response = Value::Null;

//             let resp = client.close_user_data_stream().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn close_user_data_stream_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockUserDataStreamsApiClient { force_error: true };

//             match client.close_user_data_stream().await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn keepalive_user_data_stream_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockUserDataStreamsApiClient { force_error: false };

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"listenKey":"3HBntNTepshgEdjIwSUIBgB9keLyOCg5qv3n6bYAtktG8ejcaW5HXz9Vx1JgIieg"}"#,
//             )
//             .unwrap();
//             let expected_response: models::KeepaliveUserDataStreamResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::KeepaliveUserDataStreamResponse");

//             let resp = client.keepalive_user_data_stream().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn keepalive_user_data_stream_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockUserDataStreamsApiClient { force_error: false };

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"listenKey":"3HBntNTepshgEdjIwSUIBgB9keLyOCg5qv3n6bYAtktG8ejcaW5HXz9Vx1JgIieg"}"#,
//             )
//             .unwrap();
//             let expected_response: models::KeepaliveUserDataStreamResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::KeepaliveUserDataStreamResponse");

//             let resp = client.keepalive_user_data_stream().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn keepalive_user_data_stream_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockUserDataStreamsApiClient { force_error: true };

//             match client.keepalive_user_data_stream().await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn start_user_data_stream_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockUserDataStreamsApiClient { force_error: false };

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"listenKey":"pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1"}"#,
//             )
//             .unwrap();
//             let expected_response: models::StartUserDataStreamResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::StartUserDataStreamResponse");

//             let resp = client.start_user_data_stream().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn start_user_data_stream_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockUserDataStreamsApiClient { force_error: false };

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"listenKey":"pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1"}"#,
//             )
//             .unwrap();
//             let expected_response: models::StartUserDataStreamResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::StartUserDataStreamResponse");

//             let resp = client.start_user_data_stream().await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn start_user_data_stream_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockUserDataStreamsApiClient { force_error: true };

//             match client.start_user_data_stream().await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }
// }
