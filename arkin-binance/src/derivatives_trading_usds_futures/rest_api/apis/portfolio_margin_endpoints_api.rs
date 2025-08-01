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
pub trait PortfolioMarginEndpointsApi: Send + Sync {
    async fn classic_portfolio_margin_account_information(
        &self,
        params: ClassicPortfolioMarginAccountInformationParams,
    ) -> anyhow::Result<RestApiResponse<models::ClassicPortfolioMarginAccountInformationResponse>>;
}

#[derive(Debug, Clone)]
pub struct PortfolioMarginEndpointsApiClient {
    configuration: ConfigurationRestApi,
}

impl PortfolioMarginEndpointsApiClient {
    pub fn new(configuration: ConfigurationRestApi) -> Self {
        Self { configuration }
    }
}

/// Request parameters for the [`classic_portfolio_margin_account_information`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`classic_portfolio_margin_account_information`](#method.classic_portfolio_margin_account_information).
#[derive(Clone, Debug, TypedBuilder)]
pub struct ClassicPortfolioMarginAccountInformationParams {
    ///
    /// The `asset` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub asset: String,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

#[async_trait]
impl PortfolioMarginEndpointsApi for PortfolioMarginEndpointsApiClient {
    async fn classic_portfolio_margin_account_information(
        &self,
        params: ClassicPortfolioMarginAccountInformationParams,
    ) -> anyhow::Result<RestApiResponse<models::ClassicPortfolioMarginAccountInformationResponse>> {
        let ClassicPortfolioMarginAccountInformationParams { asset, recv_window } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("asset".to_string(), json!(asset));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::ClassicPortfolioMarginAccountInformationResponse>(
            &self.configuration,
            "/fapi/v1/pmAccountInfo",
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

//     struct MockPortfolioMarginEndpointsApiClient {
//         force_error: bool,
//     }

//     #[async_trait]
//     impl PortfolioMarginEndpointsApi for MockPortfolioMarginEndpointsApiClient {
//         async fn classic_portfolio_margin_account_information(
//             &self,
//             _params: ClassicPortfolioMarginAccountInformationParams,
//         ) -> anyhow::Result<RestApiResponse<models::ClassicPortfolioMarginAccountInformationResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"maxWithdrawAmountUSD":"1627523.32459208","asset":"BTC","maxWithdrawAmount":"27.43689636"}"#,
//             )
//             .unwrap();
//             let dummy_response: models::ClassicPortfolioMarginAccountInformationResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::ClassicPortfolioMarginAccountInformationResponse");

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
//     fn classic_portfolio_margin_account_information_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockPortfolioMarginEndpointsApiClient { force_error: false };

//             let params = ClassicPortfolioMarginAccountInformationParams::builder("asset_example".to_string())
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"maxWithdrawAmountUSD":"1627523.32459208","asset":"BTC","maxWithdrawAmount":"27.43689636"}"#,
//             )
//             .unwrap();
//             let expected_response: models::ClassicPortfolioMarginAccountInformationResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::ClassicPortfolioMarginAccountInformationResponse");

//             let resp = client
//                 .classic_portfolio_margin_account_information(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn classic_portfolio_margin_account_information_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockPortfolioMarginEndpointsApiClient { force_error: false };

//             let params = ClassicPortfolioMarginAccountInformationParams::builder("asset_example".to_string())
//                 .recv_window(5000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"maxWithdrawAmountUSD":"1627523.32459208","asset":"BTC","maxWithdrawAmount":"27.43689636"}"#,
//             )
//             .unwrap();
//             let expected_response: models::ClassicPortfolioMarginAccountInformationResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::ClassicPortfolioMarginAccountInformationResponse");

//             let resp = client
//                 .classic_portfolio_margin_account_information(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn classic_portfolio_margin_account_information_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockPortfolioMarginEndpointsApiClient { force_error: true };

//             let params = ClassicPortfolioMarginAccountInformationParams::builder("asset_example".to_string())
//                 .build()
//                 .unwrap();

//             match client.classic_portfolio_margin_account_information(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }
// }
