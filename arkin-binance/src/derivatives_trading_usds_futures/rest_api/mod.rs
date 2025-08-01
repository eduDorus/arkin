#![allow(unused_imports)]
use http::Method;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::collections::BTreeMap;

use crate::common::{config::ConfigurationRestApi, models::RestApiResponse, utils::send_request};

mod apis;
mod models;

pub use apis::*;
pub use models::*;

#[derive(Debug, Clone)]
pub struct RestApi {
    configuration: ConfigurationRestApi,
    account_api_client: AccountApiClient,
    convert_api_client: ConvertApiClient,
    market_data_api_client: MarketDataApiClient,
    portfolio_margin_endpoints_api_client: PortfolioMarginEndpointsApiClient,
    trade_api_client: TradeApiClient,
    user_data_streams_api_client: UserDataStreamsApiClient,
}

impl RestApi {
    pub fn new(configuration: ConfigurationRestApi) -> Self {
        let account_api_client = AccountApiClient::new(configuration.clone());
        let convert_api_client = ConvertApiClient::new(configuration.clone());
        let market_data_api_client = MarketDataApiClient::new(configuration.clone());
        let portfolio_margin_endpoints_api_client = PortfolioMarginEndpointsApiClient::new(configuration.clone());
        let trade_api_client = TradeApiClient::new(configuration.clone());
        let user_data_streams_api_client = UserDataStreamsApiClient::new(configuration.clone());

        Self {
            configuration,
            account_api_client,
            convert_api_client,
            market_data_api_client,
            portfolio_margin_endpoints_api_client,
            trade_api_client,
            user_data_streams_api_client,
        }
    }

    /// Send an unsigned request to the API
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The API endpoint to send the request to
    /// * `method` - The HTTP method to use for the request
    /// * `params` - A map of parameters to send with the request
    ///
    /// # Returns
    ///
    /// A `RestApiResponse` containing the deserialized response data on success, or an error if the request fails
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Error` if the HTTP request fails or if parsing the response fails
    pub async fn send_request<R: DeserializeOwned + Send + 'static>(
        &self,
        endpoint: &str,
        method: Method,
        params: BTreeMap<String, Value>,
    ) -> anyhow::Result<RestApiResponse<R>> {
        send_request::<R>(&self.configuration, endpoint, method, params, None, false).await
    }

    /// Send a signed request to the API
    ///
    /// # Arguments
    ///
    /// * `endpoint` - The API endpoint to send the request to
    /// * `method` - The HTTP method to use for the request
    /// * `params` - A map of parameters to send with the request
    ///
    /// # Returns
    ///
    /// A `RestApiResponse` containing the deserialized response data on success, or an error if the request fails
    ///
    /// # Errors
    ///
    /// Returns an `anyhow::Error` if the HTTP request fails or if parsing the response fails
    pub async fn send_signed_request<R: DeserializeOwned + Send + 'static>(
        &self,
        endpoint: &str,
        method: Method,
        params: BTreeMap<String, Value>,
    ) -> anyhow::Result<RestApiResponse<R>> {
        send_request::<R>(&self.configuration, endpoint, method, params, None, true).await
    }

    /// Account Information `V2(USER_DATA)`
    ///
    /// Get current account information. User in single-asset/ multi-assets mode will see different value, see comments in response section for detail.
    ///
    /// Weight: 5
    ///
    /// # Arguments
    ///
    /// - `params`: [`AccountInformationV2Params`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::AccountInformationV2Response>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Account-Information-V2).
    ///
    pub async fn account_information_v2(
        &self,
        params: AccountInformationV2Params,
    ) -> anyhow::Result<RestApiResponse<models::AccountInformationV2Response>> {
        self.account_api_client.account_information_v2(params).await
    }

    /// Account Information `V3(USER_DATA)`
    ///
    /// Get current account information. User in single-asset/ multi-assets mode will see different value, see comments in response section for detail.
    ///
    /// Weight: 5
    ///
    /// # Arguments
    ///
    /// - `params`: [`AccountInformationV3Params`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::AccountInformationV3Response>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Account-Information-V3).
    ///
    pub async fn account_information_v3(
        &self,
        params: AccountInformationV3Params,
    ) -> anyhow::Result<RestApiResponse<models::AccountInformationV3Response>> {
        self.account_api_client.account_information_v3(params).await
    }

    /// Futures Account Balance V2 (`USER_DATA`)
    ///
    /// Query account balance info
    ///
    /// Weight: 5
    ///
    /// # Arguments
    ///
    /// - `params`: [`FuturesAccountBalanceV2Params`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::FuturesAccountBalanceV2ResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Futures-Account-Balance-V2).
    ///
    pub async fn futures_account_balance_v2(
        &self,
        params: FuturesAccountBalanceV2Params,
    ) -> anyhow::Result<RestApiResponse<Vec<models::FuturesAccountBalanceV2ResponseInner>>> {
        self.account_api_client.futures_account_balance_v2(params).await
    }

    /// Futures Account Balance V3 (`USER_DATA`)
    ///
    /// Query account balance info
    ///
    /// Weight: 5
    ///
    /// # Arguments
    ///
    /// - `params`: [`FuturesAccountBalanceV3Params`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::FuturesAccountBalanceV2ResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Futures-Account-Balance-V3).
    ///
    pub async fn futures_account_balance_v3(
        &self,
        params: FuturesAccountBalanceV3Params,
    ) -> anyhow::Result<RestApiResponse<Vec<models::FuturesAccountBalanceV2ResponseInner>>> {
        self.account_api_client.futures_account_balance_v3(params).await
    }

    /// Futures Account `Configuration(USER_DATA)`
    ///
    /// Query account configuration
    ///
    /// Weight: 5
    ///
    /// # Arguments
    ///
    /// - `params`: [`FuturesAccountConfigurationParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::FuturesAccountConfigurationResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Account-Config).
    ///
    pub async fn futures_account_configuration(
        &self,
        params: FuturesAccountConfigurationParams,
    ) -> anyhow::Result<RestApiResponse<models::FuturesAccountConfigurationResponse>> {
        self.account_api_client.futures_account_configuration(params).await
    }

    /// Futures Trading Quantitative Rules Indicators (`USER_DATA`)
    ///
    /// Futures trading quantitative rules indicators, for more information on this, please refer to the [Futures Trading Quantitative Rules](https://www.binance.com/en/support/faq/4f462ebe6ff445d4a170be7d9e897272)
    ///
    /// Weight: - 1 for a single symbol
    /// - 10 when the symbol parameter is omitted
    ///
    /// # Arguments
    ///
    /// - `params`: [`FuturesTradingQuantitativeRulesIndicatorsParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::FuturesTradingQuantitativeRulesIndicatorsResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Futures-Trading-Quantitative-Rules-Indicators).
    ///
    pub async fn futures_trading_quantitative_rules_indicators(
        &self,
        params: FuturesTradingQuantitativeRulesIndicatorsParams,
    ) -> anyhow::Result<RestApiResponse<models::FuturesTradingQuantitativeRulesIndicatorsResponse>> {
        self.account_api_client
            .futures_trading_quantitative_rules_indicators(params)
            .await
    }

    /// Get BNB Burn Status (`USER_DATA`)
    ///
    /// Get user's BNB Fee Discount (Fee Discount On or Fee Discount Off )
    ///
    /// Weight: 30
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetBnbBurnStatusParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::GetBnbBurnStatusResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Get-BNB-Burn-Status).
    ///
    pub async fn get_bnb_burn_status(
        &self,
        params: GetBnbBurnStatusParams,
    ) -> anyhow::Result<RestApiResponse<models::GetBnbBurnStatusResponse>> {
        self.account_api_client.get_bnb_burn_status(params).await
    }

    /// Get Current Multi-Assets Mode (`USER_DATA`)
    ///
    /// Get user's Multi-Assets mode (Multi-Assets Mode or Single-Asset Mode) on ***Every symbol***
    ///
    /// Weight: 30
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetCurrentMultiAssetsModeParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::GetCurrentMultiAssetsModeResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Get-Current-Multi-Assets-Mode).
    ///
    pub async fn get_current_multi_assets_mode(
        &self,
        params: GetCurrentMultiAssetsModeParams,
    ) -> anyhow::Result<RestApiResponse<models::GetCurrentMultiAssetsModeResponse>> {
        self.account_api_client.get_current_multi_assets_mode(params).await
    }

    /// Get Current Position `Mode(USER_DATA)`
    ///
    /// Get user's position mode (Hedge Mode or One-way Mode ) on ***EVERY symbol***
    ///
    /// Weight: 30
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetCurrentPositionModeParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::GetCurrentPositionModeResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Get-Current-Position-Mode).
    ///
    pub async fn get_current_position_mode(
        &self,
        params: GetCurrentPositionModeParams,
    ) -> anyhow::Result<RestApiResponse<models::GetCurrentPositionModeResponse>> {
        self.account_api_client.get_current_position_mode(params).await
    }

    /// Get Download Id For Futures Order History (`USER_DATA`)
    ///
    /// Get Download Id For Futures Order History
    ///
    /// * Request Limitation is 10 times per month, shared by front end download page and rest api
    /// * The time between `startTime` and `endTime` can not be longer than 1 year
    ///
    /// Weight: 1000
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetDownloadIdForFuturesOrderHistoryParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::GetDownloadIdForFuturesOrderHistoryResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Get-Download-Id-For-Futures-Order-History).
    ///
    pub async fn get_download_id_for_futures_order_history(
        &self,
        params: GetDownloadIdForFuturesOrderHistoryParams,
    ) -> anyhow::Result<RestApiResponse<models::GetDownloadIdForFuturesOrderHistoryResponse>> {
        self.account_api_client.get_download_id_for_futures_order_history(params).await
    }

    /// Get Download Id For Futures Trade History (`USER_DATA`)
    ///
    /// Get download id for futures trade history
    ///
    /// * Request Limitation is 5 times per month, shared by front end download page and rest api
    /// * The time between `startTime` and `endTime` can not be longer than 1 year
    ///
    /// Weight: 1000
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetDownloadIdForFuturesTradeHistoryParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::GetDownloadIdForFuturesTradeHistoryResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Get-Download-Id-For-Futures-Trade-History).
    ///
    pub async fn get_download_id_for_futures_trade_history(
        &self,
        params: GetDownloadIdForFuturesTradeHistoryParams,
    ) -> anyhow::Result<RestApiResponse<models::GetDownloadIdForFuturesTradeHistoryResponse>> {
        self.account_api_client.get_download_id_for_futures_trade_history(params).await
    }

    /// Get Download Id For Futures Transaction `History(USER_DATA)`
    ///
    /// Get download id for futures transaction history
    ///
    /// * Request Limitation is 5 times per month, shared by front end download page and rest api
    /// * The time between `startTime` and `endTime` can not be longer than 1 year
    ///
    /// Weight: 1000
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetDownloadIdForFuturesTransactionHistoryParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::GetDownloadIdForFuturesTransactionHistoryResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Get-Download-Id-For-Futures-Transaction-History).
    ///
    pub async fn get_download_id_for_futures_transaction_history(
        &self,
        params: GetDownloadIdForFuturesTransactionHistoryParams,
    ) -> anyhow::Result<RestApiResponse<models::GetDownloadIdForFuturesTransactionHistoryResponse>> {
        self.account_api_client
            .get_download_id_for_futures_transaction_history(params)
            .await
    }

    /// Get Futures Order History Download Link by Id (`USER_DATA`)
    ///
    /// Get futures order history download link by Id
    ///
    /// * Download link expiration: 24h
    ///
    /// Weight: 10
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetFuturesOrderHistoryDownloadLinkByIdParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::GetFuturesOrderHistoryDownloadLinkByIdResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Get-Futures-Order-History-Download-Link-by-Id).
    ///
    pub async fn get_futures_order_history_download_link_by_id(
        &self,
        params: GetFuturesOrderHistoryDownloadLinkByIdParams,
    ) -> anyhow::Result<RestApiResponse<models::GetFuturesOrderHistoryDownloadLinkByIdResponse>> {
        self.account_api_client
            .get_futures_order_history_download_link_by_id(params)
            .await
    }

    /// Get Futures Trade Download Link by `Id(USER_DATA)`
    ///
    /// Get futures trade download link by Id
    ///
    /// * Download link expiration: 24h
    ///
    /// Weight: 10
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetFuturesTradeDownloadLinkByIdParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::GetFuturesTradeDownloadLinkByIdResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Get-Futures-Trade-Download-Link-by-Id).
    ///
    pub async fn get_futures_trade_download_link_by_id(
        &self,
        params: GetFuturesTradeDownloadLinkByIdParams,
    ) -> anyhow::Result<RestApiResponse<models::GetFuturesTradeDownloadLinkByIdResponse>> {
        self.account_api_client.get_futures_trade_download_link_by_id(params).await
    }

    /// Get Futures Transaction History Download Link by Id (`USER_DATA`)
    ///
    /// Get futures transaction history download link by Id
    ///
    /// * Download link expiration: 24h
    ///
    /// Weight: 10
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetFuturesTransactionHistoryDownloadLinkByIdParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::GetFuturesTransactionHistoryDownloadLinkByIdResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Get-Futures-Transaction-History-Download-Link-by-Id).
    ///
    pub async fn get_futures_transaction_history_download_link_by_id(
        &self,
        params: GetFuturesTransactionHistoryDownloadLinkByIdParams,
    ) -> anyhow::Result<RestApiResponse<models::GetFuturesTransactionHistoryDownloadLinkByIdResponse>> {
        self.account_api_client
            .get_futures_transaction_history_download_link_by_id(params)
            .await
    }

    /// Get Income History (`USER_DATA`)
    ///
    /// Query income history
    ///
    /// * If neither `startTime` nor `endTime` is sent, the recent 7-day data will be returned.
    /// * If `incomeType ` is not sent, all kinds of flow will be returned
    /// * "trandId" is unique in the same incomeType for a user
    /// * Income history only contains data for the last three months
    ///
    /// Weight: 30
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetIncomeHistoryParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::GetIncomeHistoryResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Get-Income-History).
    ///
    pub async fn get_income_history(
        &self,
        params: GetIncomeHistoryParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetIncomeHistoryResponseInner>>> {
        self.account_api_client.get_income_history(params).await
    }

    /// Notional and Leverage Brackets (`USER_DATA`)
    ///
    /// Query user notional and leverage bracket on speicfic symbol
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`NotionalAndLeverageBracketsParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::NotionalAndLeverageBracketsResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Notional-and-Leverage-Brackets).
    ///
    pub async fn notional_and_leverage_brackets(
        &self,
        params: NotionalAndLeverageBracketsParams,
    ) -> anyhow::Result<RestApiResponse<models::NotionalAndLeverageBracketsResponse>> {
        self.account_api_client.notional_and_leverage_brackets(params).await
    }

    /// Query User Rate Limit (`USER_DATA`)
    ///
    /// Query User Rate Limit
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`QueryUserRateLimitParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::QueryUserRateLimitResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Query-Rate-Limit).
    ///
    pub async fn query_user_rate_limit(
        &self,
        params: QueryUserRateLimitParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::QueryUserRateLimitResponseInner>>> {
        self.account_api_client.query_user_rate_limit(params).await
    }

    /// Symbol `Configuration(USER_DATA)`
    ///
    /// Get current account symbol configuration.
    ///
    /// Weight: 5
    ///
    /// # Arguments
    ///
    /// - `params`: [`SymbolConfigurationParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::SymbolConfigurationResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Symbol-Config).
    ///
    pub async fn symbol_configuration(
        &self,
        params: SymbolConfigurationParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::SymbolConfigurationResponseInner>>> {
        self.account_api_client.symbol_configuration(params).await
    }

    /// Toggle BNB Burn On Futures Trade (TRADE)
    ///
    /// Change user's BNB Fee Discount (Fee Discount On or Fee Discount Off ) on ***EVERY symbol***
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`ToggleBnbBurnOnFuturesTradeParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::ToggleBnbBurnOnFuturesTradeResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/Toggle-BNB-Burn-On-Futures-Trade).
    ///
    pub async fn toggle_bnb_burn_on_futures_trade(
        &self,
        params: ToggleBnbBurnOnFuturesTradeParams,
    ) -> anyhow::Result<RestApiResponse<models::ToggleBnbBurnOnFuturesTradeResponse>> {
        self.account_api_client.toggle_bnb_burn_on_futures_trade(params).await
    }

    /// User Commission Rate (`USER_DATA`)
    ///
    /// Get User Commission Rate
    ///
    /// Weight: 20
    ///
    /// # Arguments
    ///
    /// - `params`: [`UserCommissionRateParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::UserCommissionRateResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/account/rest-api/User-Commission-Rate).
    ///
    pub async fn user_commission_rate(
        &self,
        params: UserCommissionRateParams,
    ) -> anyhow::Result<RestApiResponse<models::UserCommissionRateResponse>> {
        self.account_api_client.user_commission_rate(params).await
    }

    /// Accept the offered quote (`USER_DATA`)
    ///
    /// Accept the offered quote by quote ID.
    ///
    /// Weight: 200(IP)
    ///
    /// # Arguments
    ///
    /// - `params`: [`AcceptTheOfferedQuoteParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::AcceptTheOfferedQuoteResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/convert/Accept-Quote).
    ///
    pub async fn accept_the_offered_quote(
        &self,
        params: AcceptTheOfferedQuoteParams,
    ) -> anyhow::Result<RestApiResponse<models::AcceptTheOfferedQuoteResponse>> {
        self.convert_api_client.accept_the_offered_quote(params).await
    }

    /// List All Convert Pairs
    ///
    /// Query for all convertible token pairs and the tokens’ respective upper/lower limits
    ///
    /// * User needs to supply either or both of the input parameter
    /// * If not defined for both fromAsset and toAsset, only partial token pairs will be returned
    /// * Asset BNFCR is only available to convert for MICA region users.
    ///
    /// Weight: 20(IP)
    ///
    /// # Arguments
    ///
    /// - `params`: [`ListAllConvertPairsParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::ListAllConvertPairsResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/convert/).
    ///
    pub async fn list_all_convert_pairs(
        &self,
        params: ListAllConvertPairsParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::ListAllConvertPairsResponseInner>>> {
        self.convert_api_client.list_all_convert_pairs(params).await
    }

    /// Order `status(USER_DATA)`
    ///
    /// Query order status by order ID.
    ///
    /// Weight: 50(IP)
    ///
    /// # Arguments
    ///
    /// - `params`: [`OrderStatusParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::OrderStatusResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/convert/Order-Status).
    ///
    pub async fn order_status(
        &self,
        params: OrderStatusParams,
    ) -> anyhow::Result<RestApiResponse<models::OrderStatusResponse>> {
        self.convert_api_client.order_status(params).await
    }

    /// Send Quote `Request(USER_DATA)`
    ///
    /// Request a quote for the requested token pairs
    ///
    /// * Either fromAmount or toAmount should be sent
    /// * `quoteId` will be returned only if you have enough funds to convert
    ///
    /// Weight: 50(IP)
    ///
    /// # Arguments
    ///
    /// - `params`: [`SendQuoteRequestParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::SendQuoteRequestResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/convert/Send-quote-request).
    ///
    pub async fn send_quote_request(
        &self,
        params: SendQuoteRequestParams,
    ) -> anyhow::Result<RestApiResponse<models::SendQuoteRequestResponse>> {
        self.convert_api_client.send_quote_request(params).await
    }

    /// Basis
    ///
    /// Query future basis
    ///
    /// * If startTime and endTime are not sent, the most recent data is returned.
    /// * Only the data of the latest 30 days is available.
    ///
    /// Weight: 0
    ///
    /// # Arguments
    ///
    /// - `params`: [`BasisParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::BasisResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Basis).
    ///
    pub async fn basis(&self, params: BasisParams) -> anyhow::Result<RestApiResponse<Vec<models::BasisResponseInner>>> {
        self.market_data_api_client.basis(params).await
    }

    /// Check Server Time
    ///
    /// Test connectivity to the Rest API and get the current server time.
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`CheckServerTimeParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::CheckServerTimeResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Check-Server-Time).
    ///
    pub async fn check_server_time(&self) -> anyhow::Result<RestApiResponse<models::CheckServerTimeResponse>> {
        self.market_data_api_client.check_server_time().await
    }

    /// Composite Index Symbol Information
    ///
    /// Query composite index symbol information
    ///
    /// * Only for composite index symbols
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`CompositeIndexSymbolInformationParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::CompositeIndexSymbolInformationResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Composite-Index-Symbol-Information).
    ///
    pub async fn composite_index_symbol_information(
        &self,
        params: CompositeIndexSymbolInformationParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::CompositeIndexSymbolInformationResponseInner>>> {
        self.market_data_api_client.composite_index_symbol_information(params).await
    }

    /// Compressed/Aggregate Trades List
    ///
    /// Get compressed, aggregate market trades. Market trades that fill in 100ms with the same price and the same taking side will have the quantity aggregated.
    ///
    ///
    /// * support querying futures trade histories that are not older than one year
    /// * If both `startTime` and `endTime` are sent, time between `startTime` and `endTime` must be less than 1 hour.
    /// * If `fromId`, `startTime`, and `endTime` are not sent, the most recent aggregate trades will be returned.
    /// * Only market trades will be aggregated and returned, which means the insurance fund trades and ADL trades won't be aggregated.
    /// * Sending both `startTime`/`endTime` and `fromId` might cause response timeout, please send either `fromId` or `startTime`/`endTime`
    ///
    /// Weight: 20
    ///
    /// # Arguments
    ///
    /// - `params`: [`CompressedAggregateTradesListParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::CompressedAggregateTradesListResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Compressed-Aggregate-Trades-List).
    ///
    pub async fn compressed_aggregate_trades_list(
        &self,
        params: CompressedAggregateTradesListParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::CompressedAggregateTradesListResponseInner>>> {
        self.market_data_api_client.compressed_aggregate_trades_list(params).await
    }

    /// Continuous Contract Kline/Candlestick Data
    ///
    /// Kline/candlestick bars for a specific contract type.
    /// Klines are uniquely identified by their open time.
    ///
    /// * If startTime and endTime are not sent, the most recent klines are returned.
    /// * Contract type:
    /// * PERPETUAL
    /// * `CURRENT_QUARTER`
    /// * `NEXT_QUARTER`
    ///
    /// Weight: based on parameter LIMIT
    /// | LIMIT       | weight |
    /// | ----------- | ------ |
    /// | [1,100)     | 1      |
    /// | [100, 500)  | 2      |
    /// | [500, 1000] | 5      |
    /// | > 1000      | 10     |
    ///
    /// # Arguments
    ///
    /// - `params`: [`ContinuousContractKlineCandlestickDataParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<Vec<models::ContinuousContractKlineCandlestickDataResponseItemInner>>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Continuous-Contract-Kline-Candlestick-Data).
    ///
    pub async fn continuous_contract_kline_candlestick_data(
        &self,
        params: ContinuousContractKlineCandlestickDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::ContinuousContractKlineCandlestickDataResponseItemInner>>>>
    {
        self.market_data_api_client
            .continuous_contract_kline_candlestick_data(params)
            .await
    }

    /// Exchange Information
    ///
    /// Current exchange trading rules and symbol information
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`ExchangeInformationParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::ExchangeInformationResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Exchange-Information).
    ///
    pub async fn exchange_information(&self) -> anyhow::Result<RestApiResponse<models::ExchangeInformationResponse>> {
        self.market_data_api_client.exchange_information().await
    }

    /// Get Funding Rate History
    ///
    /// Get Funding Rate History
    ///
    ///
    /// * If `startTime` and `endTime` are not sent, the most recent `limit` datas are returned.
    /// * If the number of data between `startTime` and `endTime` is larger than `limit`, return as `startTime` + `limit`.
    /// * In ascending order.
    ///
    /// Weight: share 500/5min/IP rate limit with GET /fapi/v1/fundingInfo
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetFundingRateHistoryParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::GetFundingRateHistoryResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Get-Funding-Rate-History).
    ///
    pub async fn get_funding_rate_history(
        &self,
        params: GetFundingRateHistoryParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetFundingRateHistoryResponseInner>>> {
        self.market_data_api_client.get_funding_rate_history(params).await
    }

    /// Get Funding Rate Info
    ///
    /// Query funding rate info for symbols that had `FundingRateCap`/ `FundingRateFloor` / fundingIntervalHours adjustment
    ///
    /// Weight: 0
    /// share 500/5min/IP rate limit with GET /fapi/v1/fundingInfo
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetFundingRateInfoParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::GetFundingRateInfoResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Get-Funding-Rate-Info).
    ///
    pub async fn get_funding_rate_info(
        &self,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetFundingRateInfoResponseInner>>> {
        self.market_data_api_client.get_funding_rate_info().await
    }

    /// Index Price Kline/Candlestick Data
    ///
    /// Kline/candlestick bars for the index price of a pair.
    /// Klines are uniquely identified by their open time.
    ///
    ///
    /// * If startTime and endTime are not sent, the most recent klines are returned.
    ///
    /// Weight: based on parameter LIMIT
    /// | LIMIT       | weight |
    /// | ----------- | ------ |
    /// | [1,100)     | 1      |
    /// | [100, 500)  | 2      |
    /// | [500, 1000] | 5      |
    /// | > 1000      | 10     |
    ///
    /// # Arguments
    ///
    /// - `params`: [`IndexPriceKlineCandlestickDataParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<Vec<models::IndexPriceKlineCandlestickDataResponseItemInner>>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Index-Price-Kline-Candlestick-Data).
    ///
    pub async fn index_price_kline_candlestick_data(
        &self,
        params: IndexPriceKlineCandlestickDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::IndexPriceKlineCandlestickDataResponseItemInner>>>> {
        self.market_data_api_client.index_price_kline_candlestick_data(params).await
    }

    /// Kline/Candlestick Data
    ///
    /// Kline/candlestick bars for a symbol.
    /// Klines are uniquely identified by their open time.
    ///
    /// * If startTime and endTime are not sent, the most recent klines are returned.
    ///
    /// Weight: based on parameter LIMIT
    /// | LIMIT       | weight |
    /// | ----------- | ------ |
    /// | [1,100)     | 1      |
    /// | [100, 500)  | 2      |
    /// | [500, 1000] | 5      |
    /// | > 1000      | 10     |
    ///
    /// # Arguments
    ///
    /// - `params`: [`KlineCandlestickDataParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<Vec<models::KlineCandlestickDataResponseItemInner>>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Kline-Candlestick-Data).
    ///
    pub async fn kline_candlestick_data(
        &self,
        params: KlineCandlestickDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::KlineCandlestickDataResponseItemInner>>>> {
        self.market_data_api_client.kline_candlestick_data(params).await
    }

    /// Long/Short Ratio
    ///
    /// Query symbol Long/Short Ratio
    ///
    /// * If startTime and endTime are not sent, the most recent data is returned.
    /// * Only the data of the latest 30 days is available.
    /// * IP rate limit 1000 requests/5min
    ///
    /// Weight: 0
    ///
    /// # Arguments
    ///
    /// - `params`: [`LongShortRatioParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::LongShortRatioResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Long-Short-Ratio).
    ///
    pub async fn long_short_ratio(
        &self,
        params: LongShortRatioParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::LongShortRatioResponseInner>>> {
        self.market_data_api_client.long_short_ratio(params).await
    }

    /// Mark Price
    ///
    /// Mark Price and Funding Rate
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`MarkPriceParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::MarkPriceResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Mark-Price).
    ///
    pub async fn mark_price(
        &self,
        params: MarkPriceParams,
    ) -> anyhow::Result<RestApiResponse<models::MarkPriceResponse>> {
        self.market_data_api_client.mark_price(params).await
    }

    /// Mark Price Kline/Candlestick Data
    ///
    /// Kline/candlestick bars for the mark price of a symbol.
    /// Klines are uniquely identified by their open time.
    ///
    /// * If startTime and endTime are not sent, the most recent klines are returned.
    ///
    /// Weight: based on parameter LIMIT
    /// | LIMIT       | weight |
    /// | ----------- | ------ |
    /// | [1,100)     | 1      |
    /// | [100, 500)  | 2      |
    /// | [500, 1000] | 5      |
    /// | > 1000      | 10     |
    ///
    /// # Arguments
    ///
    /// - `params`: [`MarkPriceKlineCandlestickDataParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<Vec<models::MarkPriceKlineCandlestickDataResponseItemInner>>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Mark-Price-Kline-Candlestick-Data).
    ///
    pub async fn mark_price_kline_candlestick_data(
        &self,
        params: MarkPriceKlineCandlestickDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::MarkPriceKlineCandlestickDataResponseItemInner>>>> {
        self.market_data_api_client.mark_price_kline_candlestick_data(params).await
    }

    /// Multi-Assets Mode Asset Index
    ///
    /// asset index for Multi-Assets mode
    ///
    /// Weight: 1 for a single symbol; 10 when the symbol parameter is omitted
    ///
    /// # Arguments
    ///
    /// - `params`: [`MultiAssetsModeAssetIndexParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::MultiAssetsModeAssetIndexResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Multi-Assets-Mode-Asset-Index).
    ///
    pub async fn multi_assets_mode_asset_index(
        &self,
        params: MultiAssetsModeAssetIndexParams,
    ) -> anyhow::Result<RestApiResponse<models::MultiAssetsModeAssetIndexResponse>> {
        self.market_data_api_client.multi_assets_mode_asset_index(params).await
    }

    /// Old Trades Lookup (`MARKET_DATA`)
    ///
    /// Get older market historical trades.
    ///
    /// * Market trades means trades filled in the order book. Only market trades will be returned, which means the insurance fund trades and ADL trades won't be returned.
    /// * Only supports data from within the last three months
    ///
    /// Weight: 20
    ///
    /// # Arguments
    ///
    /// - `params`: [`OldTradesLookupParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::OldTradesLookupResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Old-Trades-Lookup).
    ///
    pub async fn old_trades_lookup(
        &self,
        params: OldTradesLookupParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::OldTradesLookupResponseInner>>> {
        self.market_data_api_client.old_trades_lookup(params).await
    }

    /// Open Interest
    ///
    /// Get present open interest of a specific symbol.
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`OpenInterestParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::OpenInterestResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Open-Interest).
    ///
    pub async fn open_interest(
        &self,
        params: OpenInterestParams,
    ) -> anyhow::Result<RestApiResponse<models::OpenInterestResponse>> {
        self.market_data_api_client.open_interest(params).await
    }

    /// Open Interest Statistics
    ///
    /// Open Interest Statistics
    ///
    /// * If startTime and endTime are not sent, the most recent data is returned.
    /// * Only the data of the latest 1 month is available.
    /// * IP rate limit 1000 requests/5min
    ///
    /// Weight: 0
    ///
    /// # Arguments
    ///
    /// - `params`: [`OpenInterestStatisticsParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::OpenInterestStatisticsResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Open-Interest-Statistics).
    ///
    pub async fn open_interest_statistics(
        &self,
        params: OpenInterestStatisticsParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::OpenInterestStatisticsResponseInner>>> {
        self.market_data_api_client.open_interest_statistics(params).await
    }

    /// Order Book
    ///
    /// Query symbol orderbook
    ///
    /// Weight: Adjusted based on the limit:
    /// | Limit         | Weight |
    /// | ------------- | ------ |
    /// | 5, 10, 20, 50 | 2      |
    /// | 100           | 5      |
    /// | 500           | 10     |
    /// | 1000          | 20     |
    ///
    /// # Arguments
    ///
    /// - `params`: [`OrderBookParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::OrderBookResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Order-Book).
    ///
    pub async fn order_book(
        &self,
        params: OrderBookParams,
    ) -> anyhow::Result<RestApiResponse<models::OrderBookResponse>> {
        self.market_data_api_client.order_book(params).await
    }

    /// Premium index Kline Data
    ///
    /// Premium index kline bars of a symbol. Klines are uniquely identified by their open time.
    ///
    ///
    /// * If startTime and endTime are not sent, the most recent klines are returned.
    ///
    /// Weight: based on parameter LIMIT
    /// | LIMIT       | weight |
    /// | ----------- | ------ |
    /// | [1,100)     | 1      |
    /// | [100, 500)  | 2      |
    /// | [500, 1000] | 5      |
    /// | > 1000      | 10     |
    ///
    /// # Arguments
    ///
    /// - `params`: [`PremiumIndexKlineDataParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<Vec<models::PremiumIndexKlineDataResponseItemInner>>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Premium-index-Kline-Data).
    ///
    pub async fn premium_index_kline_data(
        &self,
        params: PremiumIndexKlineDataParams,
    ) -> anyhow::Result<RestApiResponse<Vec<Vec<models::PremiumIndexKlineDataResponseItemInner>>>> {
        self.market_data_api_client.premium_index_kline_data(params).await
    }

    /// Quarterly Contract Settlement Price
    ///
    /// Latest price for a symbol or symbols.
    ///
    /// Weight: 0
    ///
    /// # Arguments
    ///
    /// - `params`: [`QuarterlyContractSettlementPriceParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::QuarterlyContractSettlementPriceResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Delivery-Price).
    ///
    pub async fn quarterly_contract_settlement_price(
        &self,
        params: QuarterlyContractSettlementPriceParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::QuarterlyContractSettlementPriceResponseInner>>> {
        self.market_data_api_client.quarterly_contract_settlement_price(params).await
    }

    /// Query Index Price Constituents
    ///
    /// Query index price constituents
    ///
    /// Weight: 2
    ///
    /// # Arguments
    ///
    /// - `params`: [`QueryIndexPriceConstituentsParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::QueryIndexPriceConstituentsResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Index-Constituents).
    ///
    pub async fn query_index_price_constituents(
        &self,
        params: QueryIndexPriceConstituentsParams,
    ) -> anyhow::Result<RestApiResponse<models::QueryIndexPriceConstituentsResponse>> {
        self.market_data_api_client.query_index_price_constituents(params).await
    }

    /// Query Insurance Fund Balance Snapshot
    ///
    /// Query Insurance Fund Balance Snapshot
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`QueryInsuranceFundBalanceSnapshotParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::QueryInsuranceFundBalanceSnapshotResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Query-Insurance-Fund-Balance-Snapshot).
    ///
    pub async fn query_insurance_fund_balance_snapshot(
        &self,
        params: QueryInsuranceFundBalanceSnapshotParams,
    ) -> anyhow::Result<RestApiResponse<models::QueryInsuranceFundBalanceSnapshotResponse>> {
        self.market_data_api_client.query_insurance_fund_balance_snapshot(params).await
    }

    /// Recent Trades List
    ///
    /// Get recent market trades
    ///
    /// * Market trades means trades filled in the order book. Only market trades will be returned, which means the insurance fund trades and ADL trades won't be returned.
    ///
    /// Weight: 5
    ///
    /// # Arguments
    ///
    /// - `params`: [`RecentTradesListParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::RecentTradesListResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Recent-Trades-List).
    ///
    pub async fn recent_trades_list(
        &self,
        params: RecentTradesListParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::RecentTradesListResponseInner>>> {
        self.market_data_api_client.recent_trades_list(params).await
    }

    /// Symbol Order Book Ticker
    ///
    /// Best price/qty on the order book for a symbol or symbols.
    ///
    /// * If the symbol is not sent, bookTickers for all symbols will be returned in an array.
    /// * The field `X-MBX-USED-WEIGHT-1M` in response header is not accurate from this endpoint, please ignore.
    ///
    /// Weight: 2 for a single symbol;
    /// 5 when the symbol parameter is omitted
    ///
    /// # Arguments
    ///
    /// - `params`: [`SymbolOrderBookTickerParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::SymbolOrderBookTickerResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Symbol-Order-Book-Ticker).
    ///
    pub async fn symbol_order_book_ticker(
        &self,
        params: SymbolOrderBookTickerParams,
    ) -> anyhow::Result<RestApiResponse<models::SymbolOrderBookTickerResponse>> {
        self.market_data_api_client.symbol_order_book_ticker(params).await
    }

    /// Symbol Price Ticker
    ///
    /// Latest price for a symbol or symbols.
    ///
    /// * If the symbol is not sent, prices for all symbols will be returned in an array.
    ///
    /// Weight: 1 for a single symbol;
    /// 2 when the symbol parameter is omitted
    ///
    /// # Arguments
    ///
    /// - `params`: [`SymbolPriceTickerParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::SymbolPriceTickerResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Symbol-Price-Ticker).
    ///
    pub async fn symbol_price_ticker(
        &self,
        params: SymbolPriceTickerParams,
    ) -> anyhow::Result<RestApiResponse<models::SymbolPriceTickerResponse>> {
        self.market_data_api_client.symbol_price_ticker(params).await
    }

    /// Symbol Price Ticker V2
    ///
    /// Latest price for a symbol or symbols.
    ///
    /// * If the symbol is not sent, prices for all symbols will be returned in an array.
    /// * The field `X-MBX-USED-WEIGHT-1M` in response header is not accurate from this endpoint, please ignore.
    ///
    /// Weight: 1 for a single symbol;
    /// 2 when the symbol parameter is omitted
    ///
    /// # Arguments
    ///
    /// - `params`: [`SymbolPriceTickerV2Params`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::SymbolPriceTickerV2Response>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Symbol-Price-Ticker-V2).
    ///
    pub async fn symbol_price_ticker_v2(
        &self,
        params: SymbolPriceTickerV2Params,
    ) -> anyhow::Result<RestApiResponse<models::SymbolPriceTickerV2Response>> {
        self.market_data_api_client.symbol_price_ticker_v2(params).await
    }

    /// Taker Buy/Sell Volume
    ///
    /// Taker Buy/Sell Volume
    ///
    /// * If startTime and endTime are not sent, the most recent data is returned.
    /// * Only the data of the latest 30 days is available.
    /// * IP rate limit 1000 requests/5min
    ///
    /// Weight: 0
    ///
    /// # Arguments
    ///
    /// - `params`: [`TakerBuySellVolumeParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::TakerBuySellVolumeResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Taker-BuySell-Volume).
    ///
    pub async fn taker_buy_sell_volume(
        &self,
        params: TakerBuySellVolumeParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::TakerBuySellVolumeResponseInner>>> {
        self.market_data_api_client.taker_buy_sell_volume(params).await
    }

    /// Test Connectivity
    ///
    /// Test connectivity to the Rest API.
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`TestConnectivityParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Value>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Test-Connectivity).
    ///
    pub async fn test_connectivity(&self) -> anyhow::Result<RestApiResponse<Value>> {
        self.market_data_api_client.test_connectivity().await
    }

    /// 24hr Ticker Price Change Statistics
    ///
    /// 24 hour rolling window price change statistics.
    /// **Careful** when accessing this with no symbol.
    ///
    /// * If the symbol is not sent, tickers for all symbols will be returned in an array.
    ///
    /// Weight: 1 for a single symbol;
    /// 40 when the symbol parameter is omitted
    ///
    /// # Arguments
    ///
    /// - `params`: [`Ticker24hrPriceChangeStatisticsParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::Ticker24hrPriceChangeStatisticsResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/24hr-Ticker-Price-Change-Statistics).
    ///
    pub async fn ticker24hr_price_change_statistics(
        &self,
        params: Ticker24hrPriceChangeStatisticsParams,
    ) -> anyhow::Result<RestApiResponse<models::Ticker24hrPriceChangeStatisticsResponse>> {
        self.market_data_api_client.ticker24hr_price_change_statistics(params).await
    }

    /// Top Trader Long/Short Ratio (Accounts)
    ///
    /// The proportion of net long and net short accounts to total accounts of the top 20% users with the highest margin balance. Each account is counted once only.
    /// Long Account % = Accounts of top traders with net long positions / Total accounts of top traders with open positions
    /// Short Account % = Accounts of top traders with net short positions / Total accounts of top traders with open positions
    /// Long/Short Ratio (Accounts) = Long Account % / Short Account %
    ///
    /// * If startTime and endTime are not sent, the most recent data is returned.
    /// * Only the data of the latest 30 days is available.
    /// * IP rate limit 1000 requests/5min
    ///
    /// Weight: 0
    ///
    /// # Arguments
    ///
    /// - `params`: [`TopTraderLongShortRatioAccountsParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::TopTraderLongShortRatioAccountsResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Top-Long-Short-Account-Ratio).
    ///
    pub async fn top_trader_long_short_ratio_accounts(
        &self,
        params: TopTraderLongShortRatioAccountsParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::TopTraderLongShortRatioAccountsResponseInner>>> {
        self.market_data_api_client.top_trader_long_short_ratio_accounts(params).await
    }

    /// Top Trader Long/Short Ratio (Positions)
    ///
    /// The proportion of net long and net short positions to total open positions of the top 20% users with the highest margin balance.
    /// Long Position % = Long positions of top traders / Total open positions of top traders
    /// Short Position % = Short positions of top traders / Total open positions of top traders
    /// Long/Short Ratio (Positions) = Long Position % / Short Position %
    ///
    /// * If startTime and endTime are not sent, the most recent data is returned.
    /// * Only the data of the latest 30 days is available.
    /// * IP rate limit 1000 requests/5min
    ///
    /// Weight: 0
    ///
    /// # Arguments
    ///
    /// - `params`: [`TopTraderLongShortRatioPositionsParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::TopTraderLongShortRatioPositionsResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Top-Trader-Long-Short-Ratio).
    ///
    pub async fn top_trader_long_short_ratio_positions(
        &self,
        params: TopTraderLongShortRatioPositionsParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::TopTraderLongShortRatioPositionsResponseInner>>> {
        self.market_data_api_client.top_trader_long_short_ratio_positions(params).await
    }

    /// Classic Portfolio Margin Account Information (`USER_DATA`)
    ///
    /// Get Classic Portfolio Margin current account information.
    ///
    ///
    /// * maxWithdrawAmount is for asset transfer out to the spot wallet.
    ///
    /// Weight: 5
    ///
    /// # Arguments
    ///
    /// - `params`: [`ClassicPortfolioMarginAccountInformationParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::ClassicPortfolioMarginAccountInformationResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/portfolio-margin-endpoints/Classic-Portfolio-Margin-Account-Information).
    ///
    pub async fn classic_portfolio_margin_account_information(
        &self,
        params: ClassicPortfolioMarginAccountInformationParams,
    ) -> anyhow::Result<RestApiResponse<models::ClassicPortfolioMarginAccountInformationResponse>> {
        self.portfolio_margin_endpoints_api_client
            .classic_portfolio_margin_account_information(params)
            .await
    }

    /// Account Trade List (`USER_DATA`)
    ///
    /// Get trades for a specific account and symbol.
    ///
    /// * If `startTime` and `endTime` are both not sent, then the last 7 days' data will be returned.
    /// * The time between `startTime` and `endTime` cannot be longer than 7 days.
    /// * The parameter `fromId` cannot be sent with `startTime` or `endTime`.
    /// * Only support querying trade in the past 6 months
    ///
    /// Weight: 5
    ///
    /// # Arguments
    ///
    /// - `params`: [`AccountTradeListParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::AccountTradeListResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Account-Trade-List).
    ///
    pub async fn account_trade_list(
        &self,
        params: AccountTradeListParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::AccountTradeListResponseInner>>> {
        self.trade_api_client.account_trade_list(params).await
    }

    /// All Orders (`USER_DATA`)
    ///
    /// Get all account orders; active, canceled, or filled.
    ///
    /// * These orders will not be found:
    /// * order status is `CANCELED` or `EXPIRED` **AND** order has NO filled trade **AND** created time + 3 days < current time
    /// * order create time + 90 days < current time
    ///
    /// * If `orderId` is set, it will get orders >= that `orderId`. Otherwise most recent orders are returned.
    /// * The query time period must be less then 7 days( default as the recent 7 days).
    ///
    /// Weight: 5
    ///
    /// # Arguments
    ///
    /// - `params`: [`AllOrdersParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::AllOrdersResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/All-Orders).
    ///
    pub async fn all_orders(
        &self,
        params: AllOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::AllOrdersResponseInner>>> {
        self.trade_api_client.all_orders(params).await
    }

    /// Auto-Cancel All Open Orders (TRADE)
    ///
    /// Cancel all open orders of the specified symbol at the end of the specified countdown.
    /// The endpoint should be called repeatedly as heartbeats so that the existing countdown time can be canceled and replaced by a new one.
    ///
    /// * Example usage:
    /// Call this endpoint at 30s intervals with an countdownTime of 120000 (120s).
    /// If this endpoint is not called within 120 seconds, all your orders of the specified symbol will be automatically canceled.
    /// If this endpoint is called with an countdownTime of 0, the countdown timer will be stopped.
    ///
    /// The system will check all countdowns **approximately every 10 milliseconds**, so please note that sufficient redundancy should be considered when using this function. We do not recommend setting the countdown time to be too precise or too small.
    ///
    /// Weight: 10
    ///
    /// # Arguments
    ///
    /// - `params`: [`AutoCancelAllOpenOrdersParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::AutoCancelAllOpenOrdersResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Auto-Cancel-All-Open-Orders).
    ///
    pub async fn auto_cancel_all_open_orders(
        &self,
        params: AutoCancelAllOpenOrdersParams,
    ) -> anyhow::Result<RestApiResponse<models::AutoCancelAllOpenOrdersResponse>> {
        self.trade_api_client.auto_cancel_all_open_orders(params).await
    }

    /// Cancel All Open Orders (TRADE)
    ///
    /// Cancel All Open Orders
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`CancelAllOpenOrdersParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::CancelAllOpenOrdersResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Cancel-All-Open-Orders).
    ///
    pub async fn cancel_all_open_orders(
        &self,
        params: CancelAllOpenOrdersParams,
    ) -> anyhow::Result<RestApiResponse<models::CancelAllOpenOrdersResponse>> {
        self.trade_api_client.cancel_all_open_orders(params).await
    }

    /// Cancel Multiple Orders (TRADE)
    ///
    /// Cancel Multiple Orders
    ///
    /// * Either `orderIdList` or `origClientOrderIdList ` must be sent.
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`CancelMultipleOrdersParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::CancelMultipleOrdersResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Cancel-Multiple-Orders).
    ///
    pub async fn cancel_multiple_orders(
        &self,
        params: CancelMultipleOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::CancelMultipleOrdersResponseInner>>> {
        self.trade_api_client.cancel_multiple_orders(params).await
    }

    /// Cancel Order (TRADE)
    ///
    /// Cancel an active order.
    ///
    /// * Either `orderId` or `origClientOrderId` must be sent.
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`CancelOrderParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::CancelOrderResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Cancel-Order).
    ///
    pub async fn cancel_order(
        &self,
        params: CancelOrderParams,
    ) -> anyhow::Result<RestApiResponse<models::CancelOrderResponse>> {
        self.trade_api_client.cancel_order(params).await
    }

    /// Change Initial Leverage(TRADE)
    ///
    /// Change user's initial leverage of specific symbol market.
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`ChangeInitialLeverageParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::ChangeInitialLeverageResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Change-Initial-Leverage).
    ///
    pub async fn change_initial_leverage(
        &self,
        params: ChangeInitialLeverageParams,
    ) -> anyhow::Result<RestApiResponse<models::ChangeInitialLeverageResponse>> {
        self.trade_api_client.change_initial_leverage(params).await
    }

    /// Change Margin Type(TRADE)
    ///
    /// Change symbol level margin type
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`ChangeMarginTypeParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::ChangeMarginTypeResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Change-Margin-Type).
    ///
    pub async fn change_margin_type(
        &self,
        params: ChangeMarginTypeParams,
    ) -> anyhow::Result<RestApiResponse<models::ChangeMarginTypeResponse>> {
        self.trade_api_client.change_margin_type(params).await
    }

    /// Change Multi-Assets Mode (TRADE)
    ///
    /// Change user's Multi-Assets mode (Multi-Assets Mode or Single-Asset Mode) on ***Every symbol***
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`ChangeMultiAssetsModeParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::ChangeMultiAssetsModeResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Change-Multi-Assets-Mode).
    ///
    pub async fn change_multi_assets_mode(
        &self,
        params: ChangeMultiAssetsModeParams,
    ) -> anyhow::Result<RestApiResponse<models::ChangeMultiAssetsModeResponse>> {
        self.trade_api_client.change_multi_assets_mode(params).await
    }

    /// Change Position Mode(TRADE)
    ///
    /// Change user's position mode (Hedge Mode or One-way Mode ) on ***EVERY symbol***
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`ChangePositionModeParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::ChangePositionModeResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Change-Position-Mode).
    ///
    pub async fn change_position_mode(
        &self,
        params: ChangePositionModeParams,
    ) -> anyhow::Result<RestApiResponse<models::ChangePositionModeResponse>> {
        self.trade_api_client.change_position_mode(params).await
    }

    /// Current All Open Orders (`USER_DATA`)
    ///
    /// Get all open orders on a symbol.
    ///
    /// * If the symbol is not sent, orders for all symbols will be returned in an array.
    ///
    /// Weight: 1 for a single symbol; 40 when the symbol parameter is omitted
    /// Careful when accessing this with no symbol.
    ///
    /// # Arguments
    ///
    /// - `params`: [`CurrentAllOpenOrdersParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::AllOrdersResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Current-All-Open-Orders).
    ///
    pub async fn current_all_open_orders(
        &self,
        params: CurrentAllOpenOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::AllOrdersResponseInner>>> {
        self.trade_api_client.current_all_open_orders(params).await
    }

    /// Get Order Modify History (`USER_DATA`)
    ///
    /// Get order modification history
    ///
    /// * Either `orderId` or `origClientOrderId` must be sent, and the `orderId` will prevail if both are sent.
    /// * Order modify history longer than 3 month is not avaliable
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetOrderModifyHistoryParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::GetOrderModifyHistoryResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Get-Order-Modify-History).
    ///
    pub async fn get_order_modify_history(
        &self,
        params: GetOrderModifyHistoryParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetOrderModifyHistoryResponseInner>>> {
        self.trade_api_client.get_order_modify_history(params).await
    }

    /// Get Position Margin Change History (TRADE)
    ///
    /// Get Position Margin Change History
    ///
    /// * Support querying future histories that are not older than 30 days
    /// * The time between `startTime` and `endTime`can't be more than 30 days
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`GetPositionMarginChangeHistoryParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::GetPositionMarginChangeHistoryResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Get-Position-Margin-Change-History).
    ///
    pub async fn get_position_margin_change_history(
        &self,
        params: GetPositionMarginChangeHistoryParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetPositionMarginChangeHistoryResponseInner>>> {
        self.trade_api_client.get_position_margin_change_history(params).await
    }

    /// Modify Isolated Position Margin(TRADE)
    ///
    /// Modify Isolated Position Margin
    ///
    ///
    /// * Only for isolated symbol
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`ModifyIsolatedPositionMarginParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::ModifyIsolatedPositionMarginResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Modify-Isolated-Position-Margin).
    ///
    pub async fn modify_isolated_position_margin(
        &self,
        params: ModifyIsolatedPositionMarginParams,
    ) -> anyhow::Result<RestApiResponse<models::ModifyIsolatedPositionMarginResponse>> {
        self.trade_api_client.modify_isolated_position_margin(params).await
    }

    /// Modify Multiple Orders(TRADE)
    ///
    /// Modify Multiple Orders (TRADE)
    ///
    /// * Parameter rules are same with `Modify Order`
    /// * Batch modify orders are processed concurrently, and the order of matching is not guaranteed.
    /// * The order of returned contents for batch modify orders is the same as the order of the order list.
    /// * One order can only be modfied for less than 10000 times
    ///
    /// Weight: 5 on 10s order rate limit(X-MBX-ORDER-COUNT-10S);
    /// 1 on 1min order rate limit(X-MBX-ORDER-COUNT-1M);
    /// 5 on IP rate limit(x-mbx-used-weight-1m);
    ///
    /// # Arguments
    ///
    /// - `params`: [`ModifyMultipleOrdersParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::ModifyMultipleOrdersResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Modify-Multiple-Orders).
    ///
    pub async fn modify_multiple_orders(
        &self,
        params: ModifyMultipleOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::ModifyMultipleOrdersResponseInner>>> {
        self.trade_api_client.modify_multiple_orders(params).await
    }

    /// Modify Order (TRADE)
    ///
    /// Order modify function, currently only LIMIT order modification is supported, modified orders will be reordered in the match queue
    ///
    ///
    /// * Either `orderId` or `origClientOrderId` must be sent, and the `orderId` will prevail if both are sent.
    /// * Both `quantity` and `price` must be sent, which is different from dapi modify order endpoint.
    /// * When the new `quantity` or `price` doesn't satisfy `PRICE_FILTER` / `PERCENT_FILTER` / `LOT_SIZE`, amendment will be rejected and the order will stay as it is.
    /// * However the order will be cancelled by the amendment in the following situations:
    /// * when the order is in partially filled status and the new `quantity` <= `executedQty`
    /// * When the order is `GTX` and the new price will cause it to be executed immediately
    /// * One order can only be modfied for less than 10000 times
    ///
    /// Weight: 1 on 10s order rate limit(X-MBX-ORDER-COUNT-10S);
    /// 1 on 1min order rate limit(X-MBX-ORDER-COUNT-1M);
    /// 1 on IP rate limit(x-mbx-used-weight-1m)
    ///
    /// # Arguments
    ///
    /// - `params`: [`ModifyOrderParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::ModifyOrderResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Modify-Order).
    ///
    pub async fn modify_order(
        &self,
        params: ModifyOrderParams,
    ) -> anyhow::Result<RestApiResponse<models::ModifyOrderResponse>> {
        self.trade_api_client.modify_order(params).await
    }

    /// New Order(TRADE)
    ///
    /// Send in a new order.
    ///
    /// * Order with type `STOP`,  parameter `timeInForce` can be sent ( default `GTC`).
    /// * Order with type `TAKE_PROFIT`,  parameter `timeInForce` can be sent ( default `GTC`).
    /// * Condition orders will be triggered when:
    ///
    /// * If parameter`priceProtect`is sent as true:
    /// * when price reaches the `stopPrice` ，the difference rate between "`MARK_PRICE`" and "`CONTRACT_PRICE`" cannot be larger than the "triggerProtect" of the symbol
    /// * "triggerProtect" of a symbol can be got from `GET /fapi/v1/exchangeInfo`
    ///
    /// * `STOP`, `STOP_MARKET`:
    /// * BUY: latest price ("`MARK_PRICE`" or "`CONTRACT_PRICE`") >= `stopPrice`
    /// * SELL: latest price ("`MARK_PRICE`" or "`CONTRACT_PRICE`") <= `stopPrice`
    /// * `TAKE_PROFIT`, `TAKE_PROFIT_MARKET`:
    /// * BUY: latest price ("`MARK_PRICE`" or "`CONTRACT_PRICE`") <= `stopPrice`
    /// * SELL: latest price ("`MARK_PRICE`" or "`CONTRACT_PRICE`") >= `stopPrice`
    /// * `TRAILING_STOP_MARKET`:
    /// * BUY: the lowest price after order placed `<= `activationPrice`, and the latest price >`= the lowest price * (1 + `callbackRate`)
    /// * SELL: the highest price after order placed >= `activationPrice`, and the latest price <= the highest price * (1 - `callbackRate`)
    ///
    /// * For `TRAILING_STOP_MARKET`, if you got such error code.
    /// ``{"code": -2021, "msg": "Order would immediately trigger."}``
    /// means that the parameters you send do not meet the following requirements:
    /// * BUY: `activationPrice` should be smaller than latest price.
    /// * SELL: `activationPrice` should be larger than latest price.
    ///
    /// * If `newOrderRespType ` is sent as `RESULT` :
    /// * `MARKET` order: the final FILLED result of the order will be return directly.
    /// * `LIMIT` order with special `timeInForce`: the final status result of the order(FILLED or EXPIRED) will be returned directly.
    ///
    /// * `STOP_MARKET`, `TAKE_PROFIT_MARKET` with `closePosition`=`true`:
    /// * Follow the same rules for condition orders.
    /// * If triggered，**close all** current long position( if `SELL`) or current short position( if `BUY`).
    /// * Cannot be used with `quantity` paremeter
    /// * Cannot be used with `reduceOnly` parameter
    /// * In Hedge Mode,cannot be used with `BUY` orders in `LONG` position side. and cannot be used with `SELL` orders in `SHORT` position side
    /// * `selfTradePreventionMode` is only effective when `timeInForce` set to `IOC` or `GTC` or `GTD`.
    /// * In extreme market conditions, timeInForce `GTD` order auto cancel time might be delayed comparing to `goodTillDate`
    ///
    /// Weight: 1 on 10s order rate limit(X-MBX-ORDER-COUNT-10S);
    /// 1 on 1min order rate limit(X-MBX-ORDER-COUNT-1M);
    /// 0 on IP rate limit(x-mbx-used-weight-1m)
    ///
    /// # Arguments
    ///
    /// - `params`: [`NewOrderParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::NewOrderResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/New-Order).
    ///
    pub async fn new_order(&self, params: NewOrderParams) -> anyhow::Result<RestApiResponse<models::NewOrderResponse>> {
        self.trade_api_client.new_order(params).await
    }

    /// Place Multiple Orders(TRADE)
    ///
    /// Place Multiple Orders
    ///
    /// * Paremeter rules are same with `New Order`
    /// * Batch orders are processed concurrently, and the order of matching is not guaranteed.
    /// * The order of returned contents for batch orders is the same as the order of the order list.
    ///
    /// Weight: 5 on 10s order rate limit(X-MBX-ORDER-COUNT-10S);
    /// 1 on 1min order rate limit(X-MBX-ORDER-COUNT-1M);
    /// 5 on IP rate limit(x-mbx-used-weight-1m);
    ///
    /// # Arguments
    ///
    /// - `params`: [`PlaceMultipleOrdersParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::PlaceMultipleOrdersResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Place-Multiple-Orders).
    ///
    pub async fn place_multiple_orders(
        &self,
        params: PlaceMultipleOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::PlaceMultipleOrdersResponseInner>>> {
        self.trade_api_client.place_multiple_orders(params).await
    }

    /// Position ADL Quantile `Estimation(USER_DATA)`
    ///
    /// Position ADL Quantile Estimation
    ///
    /// * Values update every 30s.
    /// * Values 0, 1, 2, 3, 4 shows the queue position and possibility of ADL from low to high.
    /// * For positions of the symbol are in One-way Mode or isolated margined in Hedge Mode, "LONG", "SHORT", and "BOTH" will be returned to show the positions' adl quantiles of different position sides.
    /// * If the positions of the symbol are crossed margined in Hedge Mode:
    /// * "HEDGE" as a sign will be returned instead of "BOTH";
    /// * A same value caculated on unrealized pnls on long and short sides' positions will be shown for "LONG" and "SHORT" when there are positions in both of long and short sides.
    ///
    /// Weight: 5
    ///
    /// # Arguments
    ///
    /// - `params`: [`PositionAdlQuantileEstimationParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::PositionAdlQuantileEstimationResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Position-ADL-Quantile-Estimation).
    ///
    pub async fn position_adl_quantile_estimation(
        &self,
        params: PositionAdlQuantileEstimationParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::PositionAdlQuantileEstimationResponseInner>>> {
        self.trade_api_client.position_adl_quantile_estimation(params).await
    }

    /// Position Information V2 (`USER_DATA`)
    ///
    /// Get current position information.
    ///
    /// Please use with user data stream `ACCOUNT_UPDATE` to meet your timeliness and accuracy needs.
    ///
    /// Weight: 5
    ///
    /// # Arguments
    ///
    /// - `params`: [`PositionInformationV2Params`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::PositionInformationV2ResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Position-Information-V2).
    ///
    pub async fn position_information_v2(
        &self,
        params: PositionInformationV2Params,
    ) -> anyhow::Result<RestApiResponse<Vec<models::PositionInformationV2ResponseInner>>> {
        self.trade_api_client.position_information_v2(params).await
    }

    /// Position Information V3 (`USER_DATA`)
    ///
    /// Get current position information(only symbol that has position or open orders will be returned).
    ///
    /// Please use with user data stream `ACCOUNT_UPDATE` to meet your timeliness and accuracy needs.
    ///
    /// Weight: 5
    ///
    /// # Arguments
    ///
    /// - `params`: [`PositionInformationV3Params`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::PositionInformationV3ResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Position-Information-V3).
    ///
    pub async fn position_information_v3(
        &self,
        params: PositionInformationV3Params,
    ) -> anyhow::Result<RestApiResponse<Vec<models::PositionInformationV3ResponseInner>>> {
        self.trade_api_client.position_information_v3(params).await
    }

    /// Query Current Open Order (`USER_DATA`)
    ///
    /// Query open order
    ///
    ///
    /// * Either`orderId` or `origClientOrderId` must be sent
    /// * If the queried order has been filled or cancelled, the error message "Order does not exist" will be returned.
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`QueryCurrentOpenOrderParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::QueryCurrentOpenOrderResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Query-Current-Open-Order).
    ///
    pub async fn query_current_open_order(
        &self,
        params: QueryCurrentOpenOrderParams,
    ) -> anyhow::Result<RestApiResponse<models::QueryCurrentOpenOrderResponse>> {
        self.trade_api_client.query_current_open_order(params).await
    }

    /// Query Order (`USER_DATA`)
    ///
    /// Check an order's status.
    ///
    /// * These orders will not be found:
    /// * order status is `CANCELED` or `EXPIRED` **AND** order has NO filled trade **AND** created time + 3 days < current time
    /// * order create time + 90 days < current time
    ///
    /// * Either `orderId` or `origClientOrderId` must be sent.
    /// * `orderId` is self-increment for each specific `symbol`
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`QueryOrderParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::QueryOrderResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Query-Order).
    ///
    pub async fn query_order(
        &self,
        params: QueryOrderParams,
    ) -> anyhow::Result<RestApiResponse<models::QueryOrderResponse>> {
        self.trade_api_client.query_order(params).await
    }

    /// Test Order(TRADE)
    ///
    /// Testing order request, this order will not be submitted to matching engine
    ///
    /// * Order with type `STOP`,  parameter `timeInForce` can be sent ( default `GTC`).
    /// * Order with type `TAKE_PROFIT`,  parameter `timeInForce` can be sent ( default `GTC`).
    /// * Condition orders will be triggered when:
    ///
    /// * If parameter`priceProtect`is sent as true:
    /// * when price reaches the `stopPrice` ，the difference rate between "`MARK_PRICE`" and "`CONTRACT_PRICE`" cannot be larger than the "triggerProtect" of the symbol
    /// * "triggerProtect" of a symbol can be got from `GET /fapi/v1/exchangeInfo`
    ///
    /// * `STOP`, `STOP_MARKET`:
    /// * BUY: latest price ("`MARK_PRICE`" or "`CONTRACT_PRICE`") >= `stopPrice`
    /// * SELL: latest price ("`MARK_PRICE`" or "`CONTRACT_PRICE`") <= `stopPrice`
    /// * `TAKE_PROFIT`, `TAKE_PROFIT_MARKET`:
    /// * BUY: latest price ("`MARK_PRICE`" or "`CONTRACT_PRICE`") <= `stopPrice`
    /// * SELL: latest price ("`MARK_PRICE`" or "`CONTRACT_PRICE`") >= `stopPrice`
    /// * `TRAILING_STOP_MARKET`:
    /// * BUY: the lowest price after order placed `<= `activationPrice`, and the latest price >`= the lowest price * (1 + `callbackRate`)
    /// * SELL: the highest price after order placed >= `activationPrice`, and the latest price <= the highest price * (1 - `callbackRate`)
    ///
    /// * For `TRAILING_STOP_MARKET`, if you got such error code.
    /// ``{"code": -2021, "msg": "Order would immediately trigger."}``
    /// means that the parameters you send do not meet the following requirements:
    /// * BUY: `activationPrice` should be smaller than latest price.
    /// * SELL: `activationPrice` should be larger than latest price.
    ///
    /// * If `newOrderRespType ` is sent as `RESULT` :
    /// * `MARKET` order: the final FILLED result of the order will be return directly.
    /// * `LIMIT` order with special `timeInForce`: the final status result of the order(FILLED or EXPIRED) will be returned directly.
    ///
    /// * `STOP_MARKET`, `TAKE_PROFIT_MARKET` with `closePosition`=`true`:
    /// * Follow the same rules for condition orders.
    /// * If triggered，**close all** current long position( if `SELL`) or current short position( if `BUY`).
    /// * Cannot be used with `quantity` paremeter
    /// * Cannot be used with `reduceOnly` parameter
    /// * In Hedge Mode,cannot be used with `BUY` orders in `LONG` position side. and cannot be used with `SELL` orders in `SHORT` position side
    /// * `selfTradePreventionMode` is only effective when `timeInForce` set to `IOC` or `GTC` or `GTD`.
    /// * In extreme market conditions, timeInForce `GTD` order auto cancel time might be delayed comparing to `goodTillDate`
    ///
    /// Weight: 0
    ///
    /// # Arguments
    ///
    /// - `params`: [`TestOrderParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::TestOrderResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/New-Order-Test).
    ///
    pub async fn test_order(
        &self,
        params: TestOrderParams,
    ) -> anyhow::Result<RestApiResponse<models::TestOrderResponse>> {
        self.trade_api_client.test_order(params).await
    }

    /// User's Force Orders (`USER_DATA`)
    ///
    /// Query user's Force Orders
    ///
    /// * If "autoCloseType" is not sent, orders with both of the types will be returned
    /// * If "startTime" is not sent, data within 7 days before "endTime" can be queried
    ///
    /// Weight: 20 with symbol, 50 without symbol
    ///
    /// # Arguments
    ///
    /// - `params`: [`UsersForceOrdersParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Vec<models::UsersForceOrdersResponseInner>>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Users-Force-Orders).
    ///
    pub async fn users_force_orders(
        &self,
        params: UsersForceOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::UsersForceOrdersResponseInner>>> {
        self.trade_api_client.users_force_orders(params).await
    }

    /// Close User Data Stream (`USER_STREAM`)
    ///
    /// Close out a user data stream.
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`CloseUserDataStreamParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<Value>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/user-data-streams/Close-User-Data-Stream).
    ///
    pub async fn close_user_data_stream(&self) -> anyhow::Result<RestApiResponse<Value>> {
        self.user_data_streams_api_client.close_user_data_stream().await
    }

    /// Keepalive User Data Stream (`USER_STREAM`)
    ///
    /// Keepalive a user data stream to prevent a time out. User data streams will close after 60 minutes. It's recommended to send a ping about every 60 minutes.
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`KeepaliveUserDataStreamParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::KeepaliveUserDataStreamResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/user-data-streams/Keepalive-User-Data-Stream).
    ///
    pub async fn keepalive_user_data_stream(
        &self,
    ) -> anyhow::Result<RestApiResponse<models::KeepaliveUserDataStreamResponse>> {
        self.user_data_streams_api_client.keepalive_user_data_stream().await
    }

    /// Start User Data Stream (`USER_STREAM`)
    ///
    /// Start a new user data stream. The stream will close after 60 minutes unless a keepalive is sent. If the account has an active `listenKey`, that `listenKey` will be returned and its validity will be extended for 60 minutes.
    ///
    /// Weight: 1
    ///
    /// # Arguments
    ///
    /// - `params`: [`StartUserDataStreamParams`]
    ///   The parameters for this operation.
    ///
    /// # Returns
    ///
    /// [`RestApiResponse<models::StartUserDataStreamResponse>`] on success.
    ///
    /// # Errors
    ///
    /// This function will return an [`anyhow::Error`] if:
    /// - the HTTP request fails
    /// - any parameter is invalid
    /// - the response cannot be parsed
    /// - or one of the following occurs:
    ///   - `RequiredError`
    ///   - `ConnectorClientError`
    ///   - `UnauthorizedError`
    ///   - `ForbiddenError`
    ///   - `TooManyRequestsError`
    ///   - `RateLimitBanError`
    ///   - `ServerError`
    ///   - `NotFoundError`
    ///   - `NetworkError`
    ///   - `BadRequestError`
    ///
    ///
    /// For full API details, see the [Binance API Documentation](https://developers.binance.com/docs/derivatives/usds-margined-futures/user-data-streams/Start-User-Data-Stream).
    ///
    pub async fn start_user_data_stream(&self) -> anyhow::Result<RestApiResponse<models::StartUserDataStreamResponse>> {
        self.user_data_streams_api_client.start_user_data_stream().await
    }
}
