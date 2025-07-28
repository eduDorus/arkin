#![allow(unused_imports)]
use async_trait::async_trait;
use reqwest;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use typed_builder::TypedBuilder;

use crate::common::{
    config::ConfigurationRestApi,
    models::{ParamBuildError, RestApiResponse},
    utils::send_request,
};
use crate::derivatives_trading_usds_futures::rest_api::models;

const HAS_TIME_UNIT: bool = false;

#[async_trait]
pub trait ConvertApi: Send + Sync {
    async fn accept_the_offered_quote(
        &self,
        params: AcceptTheOfferedQuoteParams,
    ) -> anyhow::Result<RestApiResponse<models::AcceptTheOfferedQuoteResponse>>;
    async fn list_all_convert_pairs(
        &self,
        params: ListAllConvertPairsParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::ListAllConvertPairsResponseInner>>>;
    async fn order_status(
        &self,
        params: OrderStatusParams,
    ) -> anyhow::Result<RestApiResponse<models::OrderStatusResponse>>;
    async fn send_quote_request(
        &self,
        params: SendQuoteRequestParams,
    ) -> anyhow::Result<RestApiResponse<models::SendQuoteRequestResponse>>;
}

#[derive(Debug, Clone)]
pub struct ConvertApiClient {
    configuration: ConfigurationRestApi,
}

impl ConvertApiClient {
    pub fn new(configuration: ConfigurationRestApi) -> Self {
        Self { configuration }
    }
}

/// Request parameters for the [`accept_the_offered_quote`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`accept_the_offered_quote`](#method.accept_the_offered_quote).
#[derive(Clone, Debug, TypedBuilder)]
pub struct AcceptTheOfferedQuoteParams {
    ///
    /// The `quote_id` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub quote_id: String,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`list_all_convert_pairs`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`list_all_convert_pairs`](#method.list_all_convert_pairs).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct ListAllConvertPairsParams {
    /// User spends coin
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub from_asset: Option<String>,
    /// User receives coin
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub to_asset: Option<String>,
}

/// Request parameters for the [`order_status`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`order_status`](#method.order_status).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct OrderStatusParams {
    /// Either orderId or quoteId is required
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub order_id: Option<String>,
    /// Either orderId or quoteId is required
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub quote_id: Option<String>,
}

/// Request parameters for the [`send_quote_request`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`send_quote_request`](#method.send_quote_request).
#[derive(Clone, Debug, TypedBuilder)]
pub struct SendQuoteRequestParams {
    ///
    /// The `from_asset` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub from_asset: String,
    ///
    /// The `to_asset` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub to_asset: String,
    /// When specified, it is the amount you will be debited after the conversion
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub from_amount: Option<rust_decimal::Decimal>,
    /// When specified, it is the amount you will be credited after the conversion
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub to_amount: Option<rust_decimal::Decimal>,
    /// 10s, default 10s
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub valid_time: Option<String>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

#[async_trait]
impl ConvertApi for ConvertApiClient {
    async fn accept_the_offered_quote(
        &self,
        params: AcceptTheOfferedQuoteParams,
    ) -> anyhow::Result<RestApiResponse<models::AcceptTheOfferedQuoteResponse>> {
        let AcceptTheOfferedQuoteParams {
            quote_id,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("quoteId".to_string(), json!(quote_id));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::AcceptTheOfferedQuoteResponse>(
            &self.configuration,
            "/fapi/v1/convert/acceptQuote",
            reqwest::Method::POST,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            true,
        )
        .await
    }

    async fn list_all_convert_pairs(
        &self,
        params: ListAllConvertPairsParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::ListAllConvertPairsResponseInner>>> {
        let ListAllConvertPairsParams {
            from_asset,
            to_asset,
        } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = from_asset {
            query_params.insert("fromAsset".to_string(), json!(rw));
        }

        if let Some(rw) = to_asset {
            query_params.insert("toAsset".to_string(), json!(rw));
        }

        send_request::<Vec<models::ListAllConvertPairsResponseInner>>(
            &self.configuration,
            "/fapi/v1/convert/exchangeInfo",
            reqwest::Method::GET,
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

    async fn order_status(
        &self,
        params: OrderStatusParams,
    ) -> anyhow::Result<RestApiResponse<models::OrderStatusResponse>> {
        let OrderStatusParams { order_id, quote_id } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = order_id {
            query_params.insert("orderId".to_string(), json!(rw));
        }

        if let Some(rw) = quote_id {
            query_params.insert("quoteId".to_string(), json!(rw));
        }

        send_request::<models::OrderStatusResponse>(
            &self.configuration,
            "/fapi/v1/convert/orderStatus",
            reqwest::Method::GET,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            true,
        )
        .await
    }

    async fn send_quote_request(
        &self,
        params: SendQuoteRequestParams,
    ) -> anyhow::Result<RestApiResponse<models::SendQuoteRequestResponse>> {
        let SendQuoteRequestParams {
            from_asset,
            to_asset,
            from_amount,
            to_amount,
            valid_time,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("fromAsset".to_string(), json!(from_asset));

        query_params.insert("toAsset".to_string(), json!(to_asset));

        if let Some(rw) = from_amount {
            query_params.insert("fromAmount".to_string(), json!(rw));
        }

        if let Some(rw) = to_amount {
            query_params.insert("toAmount".to_string(), json!(rw));
        }

        if let Some(rw) = valid_time {
            query_params.insert("validTime".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::SendQuoteRequestResponse>(
            &self.configuration,
            "/fapi/v1/convert/getQuote",
            reqwest::Method::POST,
            query_params,
            if HAS_TIME_UNIT {
                self.configuration.time_unit
            } else {
                None
            },
            true,
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

//     struct MockConvertApiClient {
//         force_error: bool,
//     }

//     #[async_trait]
//     impl ConvertApi for MockConvertApiClient {
//         async fn accept_the_offered_quote(
//             &self,
//             _params: AcceptTheOfferedQuoteParams,
//         ) -> anyhow::Result<RestApiResponse<models::AcceptTheOfferedQuoteResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"orderId":"933256278426274426","createTime":1623381330472,"orderStatus":"PROCESS"}"#,
//             )
//             .unwrap();
//             let dummy_response: models::AcceptTheOfferedQuoteResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::AcceptTheOfferedQuoteResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn list_all_convert_pairs(
//             &self,
//             _params: ListAllConvertPairsParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::ListAllConvertPairsResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"fromAsset":"BTC","toAsset":"USDT","fromAssetMinAmount":"0.0004","fromAssetMaxAmount":"50","toAssetMinAmount":"20","toAssetMaxAmount":"2500000"}]"#).unwrap();
//             let dummy_response: Vec<models::ListAllConvertPairsResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::ListAllConvertPairsResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn order_status(
//             &self,
//             _params: OrderStatusParams,
//         ) -> anyhow::Result<RestApiResponse<models::OrderStatusResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"orderId":933256278426274400,"orderStatus":"SUCCESS","fromAsset":"BTC","fromAmount":"0.00054414","toAsset":"USDT","toAmount":"20","ratio":"36755","inverseRatio":"0.00002721","createTime":1623381330472}"#).unwrap();
//             let dummy_response: models::OrderStatusResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::OrderStatusResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn send_quote_request(
//             &self,
//             _params: SendQuoteRequestParams,
//         ) -> anyhow::Result<RestApiResponse<models::SendQuoteRequestResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"quoteId":"12415572564","ratio":"38163.7","inverseRatio":"0.0000262","validTimestamp":1623319461670,"toAmount":"3816.37","fromAmount":"0.1"}"#).unwrap();
//             let dummy_response: models::SendQuoteRequestResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::SendQuoteRequestResponse");

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
//     fn accept_the_offered_quote_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockConvertApiClient { force_error: false };

//             let params = AcceptTheOfferedQuoteParams::builder("1".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"orderId":"933256278426274426","createTime":1623381330472,"orderStatus":"PROCESS"}"#,
//             )
//             .unwrap();
//             let expected_response: models::AcceptTheOfferedQuoteResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::AcceptTheOfferedQuoteResponse");

//             let resp = client.accept_the_offered_quote(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn accept_the_offered_quote_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockConvertApiClient { force_error: false };

//             let params = AcceptTheOfferedQuoteParams::builder("1".to_string())
//                 .recv_window(5000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"orderId":"933256278426274426","createTime":1623381330472,"orderStatus":"PROCESS"}"#,
//             )
//             .unwrap();
//             let expected_response: models::AcceptTheOfferedQuoteResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::AcceptTheOfferedQuoteResponse");

//             let resp = client.accept_the_offered_quote(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn accept_the_offered_quote_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockConvertApiClient { force_error: true };

//             let params = AcceptTheOfferedQuoteParams::builder("1".to_string()).build().unwrap();

//             match client.accept_the_offered_quote(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn list_all_convert_pairs_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockConvertApiClient { force_error: false };

//             let params = ListAllConvertPairsParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"fromAsset":"BTC","toAsset":"USDT","fromAssetMinAmount":"0.0004","fromAssetMaxAmount":"50","toAssetMinAmount":"20","toAssetMaxAmount":"2500000"}]"#).unwrap();
//             let expected_response : Vec<models::ListAllConvertPairsResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::ListAllConvertPairsResponseInner>");

//             let resp = client.list_all_convert_pairs(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn list_all_convert_pairs_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockConvertApiClient { force_error: false };

//             let params = ListAllConvertPairsParams::builder().from_asset("from_asset_example".to_string()).to_asset("to_asset_example".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"fromAsset":"BTC","toAsset":"USDT","fromAssetMinAmount":"0.0004","fromAssetMaxAmount":"50","toAssetMinAmount":"20","toAssetMaxAmount":"2500000"}]"#).unwrap();
//             let expected_response : Vec<models::ListAllConvertPairsResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::ListAllConvertPairsResponseInner>");

//             let resp = client.list_all_convert_pairs(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn list_all_convert_pairs_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockConvertApiClient { force_error: true };

//             let params = ListAllConvertPairsParams::builder().build().unwrap();

//             match client.list_all_convert_pairs(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn order_status_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockConvertApiClient { force_error: false };

//             let params = OrderStatusParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"orderId":933256278426274400,"orderStatus":"SUCCESS","fromAsset":"BTC","fromAmount":"0.00054414","toAsset":"USDT","toAmount":"20","ratio":"36755","inverseRatio":"0.00002721","createTime":1623381330472}"#).unwrap();
//             let expected_response : models::OrderStatusResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::OrderStatusResponse");

//             let resp = client.order_status(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn order_status_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockConvertApiClient { force_error: false };

//             let params = OrderStatusParams::builder().order_id("1".to_string()).quote_id("1".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"orderId":933256278426274400,"orderStatus":"SUCCESS","fromAsset":"BTC","fromAmount":"0.00054414","toAsset":"USDT","toAmount":"20","ratio":"36755","inverseRatio":"0.00002721","createTime":1623381330472}"#).unwrap();
//             let expected_response : models::OrderStatusResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::OrderStatusResponse");

//             let resp = client.order_status(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn order_status_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockConvertApiClient { force_error: true };

//             let params = OrderStatusParams::builder().build().unwrap();

//             match client.order_status(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn send_quote_request_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockConvertApiClient { force_error: false };

//             let params = SendQuoteRequestParams::builder("from_asset_example".to_string(),"to_asset_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"quoteId":"12415572564","ratio":"38163.7","inverseRatio":"0.0000262","validTimestamp":1623319461670,"toAmount":"3816.37","fromAmount":"0.1"}"#).unwrap();
//             let expected_response : models::SendQuoteRequestResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::SendQuoteRequestResponse");

//             let resp = client.send_quote_request(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn send_quote_request_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockConvertApiClient { force_error: false };

//             let params = SendQuoteRequestParams::builder("from_asset_example".to_string(),"to_asset_example".to_string(),).from_amount(dec!(1.0)).to_amount(dec!(1.0)).valid_time("10s".to_string()).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"quoteId":"12415572564","ratio":"38163.7","inverseRatio":"0.0000262","validTimestamp":1623319461670,"toAmount":"3816.37","fromAmount":"0.1"}"#).unwrap();
//             let expected_response : models::SendQuoteRequestResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::SendQuoteRequestResponse");

//             let resp = client.send_quote_request(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn send_quote_request_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockConvertApiClient { force_error: true };

//             let params =
//                 SendQuoteRequestParams::builder("from_asset_example".to_string(), "to_asset_example".to_string())
//                     .build()
//                     .unwrap();

//             match client.send_quote_request(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }
// }
