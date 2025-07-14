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
pub trait AccountApi: Send + Sync {
    async fn account_information_v2(
        &self,
        params: AccountInformationV2Params,
    ) -> anyhow::Result<RestApiResponse<models::AccountInformationV2Response>>;
    async fn account_information_v3(
        &self,
        params: AccountInformationV3Params,
    ) -> anyhow::Result<RestApiResponse<models::AccountInformationV3Response>>;
    async fn futures_account_balance_v2(
        &self,
        params: FuturesAccountBalanceV2Params,
    ) -> anyhow::Result<RestApiResponse<Vec<models::FuturesAccountBalanceV2ResponseInner>>>;
    async fn futures_account_balance_v3(
        &self,
        params: FuturesAccountBalanceV3Params,
    ) -> anyhow::Result<RestApiResponse<Vec<models::FuturesAccountBalanceV2ResponseInner>>>;
    async fn futures_account_configuration(
        &self,
        params: FuturesAccountConfigurationParams,
    ) -> anyhow::Result<RestApiResponse<models::FuturesAccountConfigurationResponse>>;
    async fn futures_trading_quantitative_rules_indicators(
        &self,
        params: FuturesTradingQuantitativeRulesIndicatorsParams,
    ) -> anyhow::Result<RestApiResponse<models::FuturesTradingQuantitativeRulesIndicatorsResponse>>;
    async fn get_bnb_burn_status(
        &self,
        params: GetBnbBurnStatusParams,
    ) -> anyhow::Result<RestApiResponse<models::GetBnbBurnStatusResponse>>;
    async fn get_current_multi_assets_mode(
        &self,
        params: GetCurrentMultiAssetsModeParams,
    ) -> anyhow::Result<RestApiResponse<models::GetCurrentMultiAssetsModeResponse>>;
    async fn get_current_position_mode(
        &self,
        params: GetCurrentPositionModeParams,
    ) -> anyhow::Result<RestApiResponse<models::GetCurrentPositionModeResponse>>;
    async fn get_download_id_for_futures_order_history(
        &self,
        params: GetDownloadIdForFuturesOrderHistoryParams,
    ) -> anyhow::Result<RestApiResponse<models::GetDownloadIdForFuturesOrderHistoryResponse>>;
    async fn get_download_id_for_futures_trade_history(
        &self,
        params: GetDownloadIdForFuturesTradeHistoryParams,
    ) -> anyhow::Result<RestApiResponse<models::GetDownloadIdForFuturesTradeHistoryResponse>>;
    async fn get_download_id_for_futures_transaction_history(
        &self,
        params: GetDownloadIdForFuturesTransactionHistoryParams,
    ) -> anyhow::Result<RestApiResponse<models::GetDownloadIdForFuturesTransactionHistoryResponse>>;
    async fn get_futures_order_history_download_link_by_id(
        &self,
        params: GetFuturesOrderHistoryDownloadLinkByIdParams,
    ) -> anyhow::Result<RestApiResponse<models::GetFuturesOrderHistoryDownloadLinkByIdResponse>>;
    async fn get_futures_trade_download_link_by_id(
        &self,
        params: GetFuturesTradeDownloadLinkByIdParams,
    ) -> anyhow::Result<RestApiResponse<models::GetFuturesTradeDownloadLinkByIdResponse>>;
    async fn get_futures_transaction_history_download_link_by_id(
        &self,
        params: GetFuturesTransactionHistoryDownloadLinkByIdParams,
    ) -> anyhow::Result<RestApiResponse<models::GetFuturesTransactionHistoryDownloadLinkByIdResponse>>;
    async fn get_income_history(
        &self,
        params: GetIncomeHistoryParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetIncomeHistoryResponseInner>>>;
    async fn notional_and_leverage_brackets(
        &self,
        params: NotionalAndLeverageBracketsParams,
    ) -> anyhow::Result<RestApiResponse<models::NotionalAndLeverageBracketsResponse>>;
    async fn query_user_rate_limit(
        &self,
        params: QueryUserRateLimitParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::QueryUserRateLimitResponseInner>>>;
    async fn symbol_configuration(
        &self,
        params: SymbolConfigurationParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::SymbolConfigurationResponseInner>>>;
    async fn toggle_bnb_burn_on_futures_trade(
        &self,
        params: ToggleBnbBurnOnFuturesTradeParams,
    ) -> anyhow::Result<RestApiResponse<models::ToggleBnbBurnOnFuturesTradeResponse>>;
    async fn user_commission_rate(
        &self,
        params: UserCommissionRateParams,
    ) -> anyhow::Result<RestApiResponse<models::UserCommissionRateResponse>>;
}

#[derive(Debug, Clone)]
pub struct AccountApiClient {
    configuration: ConfigurationRestApi,
}

impl AccountApiClient {
    pub fn new(configuration: ConfigurationRestApi) -> Self {
        Self { configuration }
    }
}

/// Request parameters for the [`account_information_v2`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`account_information_v2`](#method.account_information_v2).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct AccountInformationV2Params {
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`account_information_v3`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`account_information_v3`](#method.account_information_v3).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct AccountInformationV3Params {
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
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`futures_account_balance_v3`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`futures_account_balance_v3`](#method.futures_account_balance_v3).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct FuturesAccountBalanceV3Params {
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`futures_account_configuration`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`futures_account_configuration`](#method.futures_account_configuration).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct FuturesAccountConfigurationParams {
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`futures_trading_quantitative_rules_indicators`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`futures_trading_quantitative_rules_indicators`](#method.futures_trading_quantitative_rules_indicators).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct FuturesTradingQuantitativeRulesIndicatorsParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`get_bnb_burn_status`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`get_bnb_burn_status`](#method.get_bnb_burn_status).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct GetBnbBurnStatusParams {
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`get_current_multi_assets_mode`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`get_current_multi_assets_mode`](#method.get_current_multi_assets_mode).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct GetCurrentMultiAssetsModeParams {
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`get_current_position_mode`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`get_current_position_mode`](#method.get_current_position_mode).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct GetCurrentPositionModeParams {
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`get_download_id_for_futures_order_history`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`get_download_id_for_futures_order_history`](#method.get_download_id_for_futures_order_history).
#[derive(Clone, Debug, TypedBuilder)]
pub struct GetDownloadIdForFuturesOrderHistoryParams {
    /// Timestamp in ms
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub start_time: i64,
    /// Timestamp in ms
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub end_time: i64,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`get_download_id_for_futures_trade_history`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`get_download_id_for_futures_trade_history`](#method.get_download_id_for_futures_trade_history).
#[derive(Clone, Debug, TypedBuilder)]
pub struct GetDownloadIdForFuturesTradeHistoryParams {
    /// Timestamp in ms
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub start_time: i64,
    /// Timestamp in ms
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub end_time: i64,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`get_download_id_for_futures_transaction_history`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`get_download_id_for_futures_transaction_history`](#method.get_download_id_for_futures_transaction_history).
#[derive(Clone, Debug, TypedBuilder)]
pub struct GetDownloadIdForFuturesTransactionHistoryParams {
    /// Timestamp in ms
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub start_time: i64,
    /// Timestamp in ms
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub end_time: i64,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`get_futures_order_history_download_link_by_id`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`get_futures_order_history_download_link_by_id`](#method.get_futures_order_history_download_link_by_id).
#[derive(Clone, Debug, TypedBuilder)]
pub struct GetFuturesOrderHistoryDownloadLinkByIdParams {
    /// get by download id api
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub download_id: String,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`get_futures_trade_download_link_by_id`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`get_futures_trade_download_link_by_id`](#method.get_futures_trade_download_link_by_id).
#[derive(Clone, Debug, TypedBuilder)]
pub struct GetFuturesTradeDownloadLinkByIdParams {
    /// get by download id api
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub download_id: String,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`get_futures_transaction_history_download_link_by_id`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`get_futures_transaction_history_download_link_by_id`](#method.get_futures_transaction_history_download_link_by_id).
#[derive(Clone, Debug, TypedBuilder)]
pub struct GetFuturesTransactionHistoryDownloadLinkByIdParams {
    /// get by download id api
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub download_id: String,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`get_income_history`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`get_income_history`](#method.get_income_history).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct GetIncomeHistoryParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
    /// TRANSFER, `WELCOME_BONUS`, `REALIZED_PNL`, `FUNDING_FEE`, COMMISSION, `INSURANCE_CLEAR`, `REFERRAL_KICKBACK`, `COMMISSION_REBATE`, `API_REBATE`, `CONTEST_REWARD`, `CROSS_COLLATERAL_TRANSFER`, `OPTIONS_PREMIUM_FEE`, `OPTIONS_SETTLE_PROFIT`, `INTERNAL_TRANSFER`, `AUTO_EXCHANGE`, `DELIVERED_SETTELMENT`, `COIN_SWAP_DEPOSIT`, `COIN_SWAP_WITHDRAW`, `POSITION_LIMIT_INCREASE_FEE`, `STRATEGY_UMFUTURES_TRANSFER，FEE_RETURN，BFUSD_REWARD`
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub income_type: Option<String>,
    ///
    /// The `start_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub start_time: Option<i64>,
    ///
    /// The `end_time` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub end_time: Option<i64>,
    ///
    /// The `page` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub page: Option<i64>,
    /// Default 100; max 1000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub limit: Option<i64>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`notional_and_leverage_brackets`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`notional_and_leverage_brackets`](#method.notional_and_leverage_brackets).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct NotionalAndLeverageBracketsParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`query_user_rate_limit`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`query_user_rate_limit`](#method.query_user_rate_limit).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct QueryUserRateLimitParams {
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`symbol_configuration`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`symbol_configuration`](#method.symbol_configuration).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct SymbolConfigurationParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`toggle_bnb_burn_on_futures_trade`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`toggle_bnb_burn_on_futures_trade`](#method.toggle_bnb_burn_on_futures_trade).
#[derive(Clone, Debug, TypedBuilder)]
pub struct ToggleBnbBurnOnFuturesTradeParams {
    /// "true": Fee Discount On; "false": Fee Discount Off
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub fee_burn: String,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`user_commission_rate`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`user_commission_rate`](#method.user_commission_rate).
#[derive(Clone, Debug, TypedBuilder)]
pub struct UserCommissionRateParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

#[async_trait]
impl AccountApi for AccountApiClient {
    async fn account_information_v2(
        &self,
        params: AccountInformationV2Params,
    ) -> anyhow::Result<RestApiResponse<models::AccountInformationV2Response>> {
        let AccountInformationV2Params { recv_window } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::AccountInformationV2Response>(
            &self.configuration,
            "/fapi/v2/account",
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

    async fn account_information_v3(
        &self,
        params: AccountInformationV3Params,
    ) -> anyhow::Result<RestApiResponse<models::AccountInformationV3Response>> {
        let AccountInformationV3Params { recv_window } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::AccountInformationV3Response>(
            &self.configuration,
            "/fapi/v3/account",
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

    async fn futures_account_balance_v2(
        &self,
        params: FuturesAccountBalanceV2Params,
    ) -> anyhow::Result<RestApiResponse<Vec<models::FuturesAccountBalanceV2ResponseInner>>> {
        let FuturesAccountBalanceV2Params { recv_window } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<Vec<models::FuturesAccountBalanceV2ResponseInner>>(
            &self.configuration,
            "/fapi/v2/balance",
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

    async fn futures_account_balance_v3(
        &self,
        params: FuturesAccountBalanceV3Params,
    ) -> anyhow::Result<RestApiResponse<Vec<models::FuturesAccountBalanceV2ResponseInner>>> {
        let FuturesAccountBalanceV3Params { recv_window } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<Vec<models::FuturesAccountBalanceV2ResponseInner>>(
            &self.configuration,
            "/fapi/v3/balance",
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

    async fn futures_account_configuration(
        &self,
        params: FuturesAccountConfigurationParams,
    ) -> anyhow::Result<RestApiResponse<models::FuturesAccountConfigurationResponse>> {
        let FuturesAccountConfigurationParams { recv_window } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::FuturesAccountConfigurationResponse>(
            &self.configuration,
            "/fapi/v1/accountConfig",
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

    async fn futures_trading_quantitative_rules_indicators(
        &self,
        params: FuturesTradingQuantitativeRulesIndicatorsParams,
    ) -> anyhow::Result<RestApiResponse<models::FuturesTradingQuantitativeRulesIndicatorsResponse>> {
        let FuturesTradingQuantitativeRulesIndicatorsParams {
            symbol,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::FuturesTradingQuantitativeRulesIndicatorsResponse>(
            &self.configuration,
            "/fapi/v1/apiTradingStatus",
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

    async fn get_bnb_burn_status(
        &self,
        params: GetBnbBurnStatusParams,
    ) -> anyhow::Result<RestApiResponse<models::GetBnbBurnStatusResponse>> {
        let GetBnbBurnStatusParams { recv_window } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::GetBnbBurnStatusResponse>(
            &self.configuration,
            "/fapi/v1/feeBurn",
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

    async fn get_current_multi_assets_mode(
        &self,
        params: GetCurrentMultiAssetsModeParams,
    ) -> anyhow::Result<RestApiResponse<models::GetCurrentMultiAssetsModeResponse>> {
        let GetCurrentMultiAssetsModeParams { recv_window } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::GetCurrentMultiAssetsModeResponse>(
            &self.configuration,
            "/fapi/v1/multiAssetsMargin",
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

    async fn get_current_position_mode(
        &self,
        params: GetCurrentPositionModeParams,
    ) -> anyhow::Result<RestApiResponse<models::GetCurrentPositionModeResponse>> {
        let GetCurrentPositionModeParams { recv_window } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::GetCurrentPositionModeResponse>(
            &self.configuration,
            "/fapi/v1/positionSide/dual",
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

    async fn get_download_id_for_futures_order_history(
        &self,
        params: GetDownloadIdForFuturesOrderHistoryParams,
    ) -> anyhow::Result<RestApiResponse<models::GetDownloadIdForFuturesOrderHistoryResponse>> {
        let GetDownloadIdForFuturesOrderHistoryParams {
            start_time,
            end_time,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("startTime".to_string(), json!(start_time));

        query_params.insert("endTime".to_string(), json!(end_time));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::GetDownloadIdForFuturesOrderHistoryResponse>(
            &self.configuration,
            "/fapi/v1/order/asyn",
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

    async fn get_download_id_for_futures_trade_history(
        &self,
        params: GetDownloadIdForFuturesTradeHistoryParams,
    ) -> anyhow::Result<RestApiResponse<models::GetDownloadIdForFuturesTradeHistoryResponse>> {
        let GetDownloadIdForFuturesTradeHistoryParams {
            start_time,
            end_time,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("startTime".to_string(), json!(start_time));

        query_params.insert("endTime".to_string(), json!(end_time));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::GetDownloadIdForFuturesTradeHistoryResponse>(
            &self.configuration,
            "/fapi/v1/trade/asyn",
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

    async fn get_download_id_for_futures_transaction_history(
        &self,
        params: GetDownloadIdForFuturesTransactionHistoryParams,
    ) -> anyhow::Result<RestApiResponse<models::GetDownloadIdForFuturesTransactionHistoryResponse>> {
        let GetDownloadIdForFuturesTransactionHistoryParams {
            start_time,
            end_time,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("startTime".to_string(), json!(start_time));

        query_params.insert("endTime".to_string(), json!(end_time));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::GetDownloadIdForFuturesTransactionHistoryResponse>(
            &self.configuration,
            "/fapi/v1/income/asyn",
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

    async fn get_futures_order_history_download_link_by_id(
        &self,
        params: GetFuturesOrderHistoryDownloadLinkByIdParams,
    ) -> anyhow::Result<RestApiResponse<models::GetFuturesOrderHistoryDownloadLinkByIdResponse>> {
        let GetFuturesOrderHistoryDownloadLinkByIdParams {
            download_id,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("downloadId".to_string(), json!(download_id));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::GetFuturesOrderHistoryDownloadLinkByIdResponse>(
            &self.configuration,
            "/fapi/v1/order/asyn/id",
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

    async fn get_futures_trade_download_link_by_id(
        &self,
        params: GetFuturesTradeDownloadLinkByIdParams,
    ) -> anyhow::Result<RestApiResponse<models::GetFuturesTradeDownloadLinkByIdResponse>> {
        let GetFuturesTradeDownloadLinkByIdParams {
            download_id,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("downloadId".to_string(), json!(download_id));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::GetFuturesTradeDownloadLinkByIdResponse>(
            &self.configuration,
            "/fapi/v1/trade/asyn/id",
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

    async fn get_futures_transaction_history_download_link_by_id(
        &self,
        params: GetFuturesTransactionHistoryDownloadLinkByIdParams,
    ) -> anyhow::Result<RestApiResponse<models::GetFuturesTransactionHistoryDownloadLinkByIdResponse>> {
        let GetFuturesTransactionHistoryDownloadLinkByIdParams {
            download_id,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("downloadId".to_string(), json!(download_id));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::GetFuturesTransactionHistoryDownloadLinkByIdResponse>(
            &self.configuration,
            "/fapi/v1/income/asyn/id",
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

    async fn get_income_history(
        &self,
        params: GetIncomeHistoryParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetIncomeHistoryResponseInner>>> {
        let GetIncomeHistoryParams {
            symbol,
            income_type,
            start_time,
            end_time,
            page,
            limit,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        if let Some(rw) = income_type {
            query_params.insert("incomeType".to_string(), json!(rw));
        }

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        if let Some(rw) = page {
            query_params.insert("page".to_string(), json!(rw));
        }

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<Vec<models::GetIncomeHistoryResponseInner>>(
            &self.configuration,
            "/fapi/v1/income",
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

    async fn notional_and_leverage_brackets(
        &self,
        params: NotionalAndLeverageBracketsParams,
    ) -> anyhow::Result<RestApiResponse<models::NotionalAndLeverageBracketsResponse>> {
        let NotionalAndLeverageBracketsParams {
            symbol,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::NotionalAndLeverageBracketsResponse>(
            &self.configuration,
            "/fapi/v1/leverageBracket",
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

    async fn query_user_rate_limit(
        &self,
        params: QueryUserRateLimitParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::QueryUserRateLimitResponseInner>>> {
        let QueryUserRateLimitParams { recv_window } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<Vec<models::QueryUserRateLimitResponseInner>>(
            &self.configuration,
            "/fapi/v1/rateLimit/order",
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

    async fn symbol_configuration(
        &self,
        params: SymbolConfigurationParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::SymbolConfigurationResponseInner>>> {
        let SymbolConfigurationParams {
            symbol,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<Vec<models::SymbolConfigurationResponseInner>>(
            &self.configuration,
            "/fapi/v1/symbolConfig",
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

    async fn toggle_bnb_burn_on_futures_trade(
        &self,
        params: ToggleBnbBurnOnFuturesTradeParams,
    ) -> anyhow::Result<RestApiResponse<models::ToggleBnbBurnOnFuturesTradeResponse>> {
        let ToggleBnbBurnOnFuturesTradeParams {
            fee_burn,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("feeBurn".to_string(), json!(fee_burn));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::ToggleBnbBurnOnFuturesTradeResponse>(
            &self.configuration,
            "/fapi/v1/feeBurn",
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

    async fn user_commission_rate(
        &self,
        params: UserCommissionRateParams,
    ) -> anyhow::Result<RestApiResponse<models::UserCommissionRateResponse>> {
        let UserCommissionRateParams {
            symbol,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::UserCommissionRateResponse>(
            &self.configuration,
            "/fapi/v1/commissionRate",
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

//     struct MockAccountApiClient {
//         force_error: bool,
//     }

//     #[async_trait]
//     impl AccountApi for MockAccountApiClient {
//         async fn account_information_v2(
//             &self,
//             _params: AccountInformationV2Params,
//         ) -> anyhow::Result<RestApiResponse<models::AccountInformationV2Response>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"feeTier":0,"feeBurn":true,"canDeposit":true,"canWithdraw":true,"updateTime":0,"multiAssetsMargin":true,"tradeGroupId":-1,"totalInitialMargin":"0.00000000","totalMaintMargin":"0.00000000","totalWalletBalance":"126.72469206","totalUnrealizedProfit":"0.00000000","totalMarginBalance":"126.72469206","totalPositionInitialMargin":"0.00000000","totalOpenOrderInitialMargin":"0.00000000","totalCrossWalletBalance":"126.72469206","totalCrossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"126.72469206","assets":[{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1625474304765},{"asset":"BUSD","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"103.12345678","maxWithdrawAmount":"103.12345678","marginAvailable":true,"updateTime":1625474304765},{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1625474304765},{"asset":"BUSD","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"103.12345678","marginAvailable":true,"updateTime":1625474304765}],"positions":[{"symbol":"BTCUSDT","initialMargin":"0","maintMargin":"0","unrealizedProfit":"0.00000000","positionInitialMargin":"0","openOrderInitialMargin":"0","leverage":"100","isolated":true,"entryPrice":"0.00000","maxNotional":"250000","bidNotional":"0","askNotional":"0","positionSide":"BOTH","positionAmt":"0","updateTime":0},{"symbol":"BTCUSDT","initialMargin":"0","maintMargin":"0","unrealizedProfit":"0.00000000","positionInitialMargin":"0","openOrderInitialMargin":"0","leverage":"100","isolated":true,"entryPrice":"0.00000","maxNotional":"250000","bidNotional":"0","askNotional":"0","positionSide":"BOTH","positionAmt":"0","updateTime":0}],"canTrade":true}"#).unwrap();
//             let dummy_response: models::AccountInformationV2Response = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::AccountInformationV2Response");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn account_information_v3(
//             &self,
//             _params: AccountInformationV3Params,
//         ) -> anyhow::Result<RestApiResponse<models::AccountInformationV3Response>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"totalInitialMargin":"0.00000000","totalMaintMargin":"0.00000000","totalWalletBalance":"126.72469206","totalUnrealizedProfit":"0.00000000","totalMarginBalance":"126.72469206","totalPositionInitialMargin":"0.00000000","totalOpenOrderInitialMargin":"0.00000000","totalCrossWalletBalance":"126.72469206","totalCrossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"126.72469206","assets":[{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","updateTime":1625474304765},{"asset":"USDC","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"103.12345678","updateTime":1625474304765},{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1625474304765},{"asset":"BUSD","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"103.12345678","marginAvailable":true,"updateTime":1625474304765}],"positions":[{"symbol":"BTCUSDT","positionSide":"BOTH","positionAmt":"1.000","unrealizedProfit":"0.00000000","isolatedMargin":"0.00000000","notional":"0","isolatedWallet":"0","initialMargin":"0","maintMargin":"0","updateTime":0},{"symbol":"BTCUSDT","positionSide":"BOTH","positionAmt":"1.000","unrealizedProfit":"0.00000000","isolatedMargin":"0.00000000","notional":"0","isolatedWallet":"0","initialMargin":"0","maintMargin":"0","updateTime":0}]}"#).unwrap();
//             let dummy_response: models::AccountInformationV3Response = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::AccountInformationV3Response");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn futures_account_balance_v2(
//             &self,
//             _params: FuturesAccountBalanceV2Params,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::FuturesAccountBalanceV2ResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"accountAlias":"SgsR","asset":"USDT","balance":"122607.35137903","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1617939110373}]"#).unwrap();
//             let dummy_response: Vec<models::FuturesAccountBalanceV2ResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::FuturesAccountBalanceV2ResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn futures_account_balance_v3(
//             &self,
//             _params: FuturesAccountBalanceV3Params,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::FuturesAccountBalanceV2ResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"accountAlias":"SgsR","asset":"USDT","balance":"122607.35137903","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1617939110373}]"#).unwrap();
//             let dummy_response: Vec<models::FuturesAccountBalanceV2ResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::FuturesAccountBalanceV2ResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn futures_account_configuration(
//             &self,
//             _params: FuturesAccountConfigurationParams,
//         ) -> anyhow::Result<RestApiResponse<models::FuturesAccountConfigurationResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"feeTier":0,"canTrade":true,"canDeposit":true,"canWithdraw":true,"dualSidePosition":true,"updateTime":0,"multiAssetsMargin":false,"tradeGroupId":-1}"#).unwrap();
//             let dummy_response: models::FuturesAccountConfigurationResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::FuturesAccountConfigurationResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn futures_trading_quantitative_rules_indicators(
//             &self,
//             _params: FuturesTradingQuantitativeRulesIndicatorsParams,
//         ) -> anyhow::Result<RestApiResponse<models::FuturesTradingQuantitativeRulesIndicatorsResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"indicators":{"BTCUSDT":[{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"UFR","value":0.05,"triggerValue":0.995},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"IFER","value":0.99,"triggerValue":0.99},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"GCR","value":0.99,"triggerValue":0.99},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"DR","value":0.99,"triggerValue":0.99}],"ETHUSDT":[{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"UFR","value":0.05,"triggerValue":0.995},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"IFER","value":0.99,"triggerValue":0.99},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"GCR","value":0.99,"triggerValue":0.99},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"DR","value":0.99,"triggerValue":0.99}],"ACCOUNT":[{"indicator":"TMV","value":10,"triggerValue":1,"plannedRecoverTime":1644919865000,"isLocked":true}]},"updateTime":1644913304748}"#).unwrap();
//             let dummy_response: models::FuturesTradingQuantitativeRulesIndicatorsResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::FuturesTradingQuantitativeRulesIndicatorsResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_bnb_burn_status(
//             &self,
//             _params: GetBnbBurnStatusParams,
//         ) -> anyhow::Result<RestApiResponse<models::GetBnbBurnStatusResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"feeBurn":true}"#).unwrap();
//             let dummy_response: models::GetBnbBurnStatusResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::GetBnbBurnStatusResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_current_multi_assets_mode(
//             &self,
//             _params: GetCurrentMultiAssetsModeParams,
//         ) -> anyhow::Result<RestApiResponse<models::GetCurrentMultiAssetsModeResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"multiAssetsMargin":true}"#).unwrap();
//             let dummy_response: models::GetCurrentMultiAssetsModeResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::GetCurrentMultiAssetsModeResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_current_position_mode(
//             &self,
//             _params: GetCurrentPositionModeParams,
//         ) -> anyhow::Result<RestApiResponse<models::GetCurrentPositionModeResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"dualSidePosition":true}"#).unwrap();
//             let dummy_response: models::GetCurrentPositionModeResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::GetCurrentPositionModeResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_download_id_for_futures_order_history(
//             &self,
//             _params: GetDownloadIdForFuturesOrderHistoryParams,
//         ) -> anyhow::Result<RestApiResponse<models::GetDownloadIdForFuturesOrderHistoryResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"avgCostTimestampOfLast30d":7241837,"downloadId":"546975389218332672"}"#)
//                     .unwrap();
//             let dummy_response: models::GetDownloadIdForFuturesOrderHistoryResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetDownloadIdForFuturesOrderHistoryResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_download_id_for_futures_trade_history(
//             &self,
//             _params: GetDownloadIdForFuturesTradeHistoryParams,
//         ) -> anyhow::Result<RestApiResponse<models::GetDownloadIdForFuturesTradeHistoryResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"avgCostTimestampOfLast30d":7241837,"downloadId":"546975389218332672"}"#)
//                     .unwrap();
//             let dummy_response: models::GetDownloadIdForFuturesTradeHistoryResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetDownloadIdForFuturesTradeHistoryResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_download_id_for_futures_transaction_history(
//             &self,
//             _params: GetDownloadIdForFuturesTransactionHistoryParams,
//         ) -> anyhow::Result<RestApiResponse<models::GetDownloadIdForFuturesTransactionHistoryResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"avgCostTimestampOfLast30d":7241837,"downloadId":"546975389218332672"}"#)
//                     .unwrap();
//             let dummy_response: models::GetDownloadIdForFuturesTransactionHistoryResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetDownloadIdForFuturesTransactionHistoryResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_futures_order_history_download_link_by_id(
//             &self,
//             _params: GetFuturesOrderHistoryDownloadLinkByIdParams,
//         ) -> anyhow::Result<RestApiResponse<models::GetFuturesOrderHistoryDownloadLinkByIdResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"downloadId":"545923594199212032","status":"processing","url":"","notified":false,"expirationTimestamp":-1,"isExpired":null}"#).unwrap();
//             let dummy_response: models::GetFuturesOrderHistoryDownloadLinkByIdResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetFuturesOrderHistoryDownloadLinkByIdResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_futures_trade_download_link_by_id(
//             &self,
//             _params: GetFuturesTradeDownloadLinkByIdParams,
//         ) -> anyhow::Result<RestApiResponse<models::GetFuturesTradeDownloadLinkByIdResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"downloadId":"545923594199212032","status":"processing","url":"","notified":false,"expirationTimestamp":-1,"isExpired":null}"#).unwrap();
//             let dummy_response: models::GetFuturesTradeDownloadLinkByIdResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetFuturesTradeDownloadLinkByIdResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_futures_transaction_history_download_link_by_id(
//             &self,
//             _params: GetFuturesTransactionHistoryDownloadLinkByIdParams,
//         ) -> anyhow::Result<RestApiResponse<models::GetFuturesTransactionHistoryDownloadLinkByIdResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"downloadId":"545923594199212032","status":"processing","url":"","notified":false,"expirationTimestamp":-1,"isExpired":null}"#).unwrap();
//             let dummy_response: models::GetFuturesTransactionHistoryDownloadLinkByIdResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetFuturesTransactionHistoryDownloadLinkByIdResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_income_history(
//             &self,
//             _params: GetIncomeHistoryParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::GetIncomeHistoryResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"","incomeType":"TRANSFER","income":"-0.37500000","asset":"USDT","info":"TRANSFER","time":1570608000000,"tranId":9689322392,"tradeId":""},{"symbol":"BTCUSDT","incomeType":"COMMISSION","income":"-0.01000000","asset":"USDT","info":"COMMISSION","time":1570636800000,"tranId":9689322392,"tradeId":"2059192"}]"#).unwrap();
//             let dummy_response: Vec<models::GetIncomeHistoryResponseInner> = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into Vec<models::GetIncomeHistoryResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn notional_and_leverage_brackets(
//             &self,
//             _params: NotionalAndLeverageBracketsParams,
//         ) -> anyhow::Result<RestApiResponse<models::NotionalAndLeverageBracketsResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"ETHUSDT","notionalCoef":1.5,"brackets":[{"bracket":1,"initialLeverage":75,"notionalCap":10000,"notionalFloor":0,"maintMarginRatio":0.0065,"cum":0}]}]"#).unwrap();
//             let dummy_response: models::NotionalAndLeverageBracketsResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::NotionalAndLeverageBracketsResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn query_user_rate_limit(
//             &self,
//             _params: QueryUserRateLimitParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::QueryUserRateLimitResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"rateLimitType":"ORDERS","interval":"SECOND","intervalNum":10,"limit":10000},{"rateLimitType":"ORDERS","interval":"MINUTE","intervalNum":1,"limit":20000}]"#).unwrap();
//             let dummy_response: Vec<models::QueryUserRateLimitResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::QueryUserRateLimitResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn symbol_configuration(
//             &self,
//             _params: SymbolConfigurationParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::SymbolConfigurationResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","marginType":"CROSSED","isAutoAddMargin":"false","leverage":21,"maxNotionalValue":"1000000"}]"#).unwrap();
//             let dummy_response: Vec<models::SymbolConfigurationResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::SymbolConfigurationResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn toggle_bnb_burn_on_futures_trade(
//             &self,
//             _params: ToggleBnbBurnOnFuturesTradeParams,
//         ) -> anyhow::Result<RestApiResponse<models::ToggleBnbBurnOnFuturesTradeResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"code":200,"msg":"success"}"#).unwrap();
//             let dummy_response: models::ToggleBnbBurnOnFuturesTradeResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::ToggleBnbBurnOnFuturesTradeResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn user_commission_rate(
//             &self,
//             _params: UserCommissionRateParams,
//         ) -> anyhow::Result<RestApiResponse<models::UserCommissionRateResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"symbol":"BTCUSDT","makerCommissionRate":"0.0002","takerCommissionRate":"0.0004"}"#,
//             )
//             .unwrap();
//             let dummy_response: models::UserCommissionRateResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::UserCommissionRateResponse");

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
//     fn account_information_v2_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = AccountInformationV2Params::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"feeTier":0,"feeBurn":true,"canDeposit":true,"canWithdraw":true,"updateTime":0,"multiAssetsMargin":true,"tradeGroupId":-1,"totalInitialMargin":"0.00000000","totalMaintMargin":"0.00000000","totalWalletBalance":"126.72469206","totalUnrealizedProfit":"0.00000000","totalMarginBalance":"126.72469206","totalPositionInitialMargin":"0.00000000","totalOpenOrderInitialMargin":"0.00000000","totalCrossWalletBalance":"126.72469206","totalCrossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"126.72469206","assets":[{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1625474304765},{"asset":"BUSD","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"103.12345678","maxWithdrawAmount":"103.12345678","marginAvailable":true,"updateTime":1625474304765},{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1625474304765},{"asset":"BUSD","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"103.12345678","marginAvailable":true,"updateTime":1625474304765}],"positions":[{"symbol":"BTCUSDT","initialMargin":"0","maintMargin":"0","unrealizedProfit":"0.00000000","positionInitialMargin":"0","openOrderInitialMargin":"0","leverage":"100","isolated":true,"entryPrice":"0.00000","maxNotional":"250000","bidNotional":"0","askNotional":"0","positionSide":"BOTH","positionAmt":"0","updateTime":0},{"symbol":"BTCUSDT","initialMargin":"0","maintMargin":"0","unrealizedProfit":"0.00000000","positionInitialMargin":"0","openOrderInitialMargin":"0","leverage":"100","isolated":true,"entryPrice":"0.00000","maxNotional":"250000","bidNotional":"0","askNotional":"0","positionSide":"BOTH","positionAmt":"0","updateTime":0}],"canTrade":true}"#).unwrap();
//             let expected_response : models::AccountInformationV2Response = serde_json::from_value(resp_json.clone()).expect("should parse into models::AccountInformationV2Response");

//             let resp = client.account_information_v2(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn account_information_v2_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = AccountInformationV2Params::builder().recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"feeTier":0,"feeBurn":true,"canDeposit":true,"canWithdraw":true,"updateTime":0,"multiAssetsMargin":true,"tradeGroupId":-1,"totalInitialMargin":"0.00000000","totalMaintMargin":"0.00000000","totalWalletBalance":"126.72469206","totalUnrealizedProfit":"0.00000000","totalMarginBalance":"126.72469206","totalPositionInitialMargin":"0.00000000","totalOpenOrderInitialMargin":"0.00000000","totalCrossWalletBalance":"126.72469206","totalCrossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"126.72469206","assets":[{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1625474304765},{"asset":"BUSD","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"103.12345678","maxWithdrawAmount":"103.12345678","marginAvailable":true,"updateTime":1625474304765},{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1625474304765},{"asset":"BUSD","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"103.12345678","marginAvailable":true,"updateTime":1625474304765}],"positions":[{"symbol":"BTCUSDT","initialMargin":"0","maintMargin":"0","unrealizedProfit":"0.00000000","positionInitialMargin":"0","openOrderInitialMargin":"0","leverage":"100","isolated":true,"entryPrice":"0.00000","maxNotional":"250000","bidNotional":"0","askNotional":"0","positionSide":"BOTH","positionAmt":"0","updateTime":0},{"symbol":"BTCUSDT","initialMargin":"0","maintMargin":"0","unrealizedProfit":"0.00000000","positionInitialMargin":"0","openOrderInitialMargin":"0","leverage":"100","isolated":true,"entryPrice":"0.00000","maxNotional":"250000","bidNotional":"0","askNotional":"0","positionSide":"BOTH","positionAmt":"0","updateTime":0}],"canTrade":true}"#).unwrap();
//             let expected_response : models::AccountInformationV2Response = serde_json::from_value(resp_json.clone()).expect("should parse into models::AccountInformationV2Response");

//             let resp = client.account_information_v2(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn account_information_v2_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = AccountInformationV2Params::builder().build().unwrap();

//             match client.account_information_v2(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn account_information_v3_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = AccountInformationV3Params::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"totalInitialMargin":"0.00000000","totalMaintMargin":"0.00000000","totalWalletBalance":"126.72469206","totalUnrealizedProfit":"0.00000000","totalMarginBalance":"126.72469206","totalPositionInitialMargin":"0.00000000","totalOpenOrderInitialMargin":"0.00000000","totalCrossWalletBalance":"126.72469206","totalCrossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"126.72469206","assets":[{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","updateTime":1625474304765},{"asset":"USDC","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"103.12345678","updateTime":1625474304765},{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1625474304765},{"asset":"BUSD","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"103.12345678","marginAvailable":true,"updateTime":1625474304765}],"positions":[{"symbol":"BTCUSDT","positionSide":"BOTH","positionAmt":"1.000","unrealizedProfit":"0.00000000","isolatedMargin":"0.00000000","notional":"0","isolatedWallet":"0","initialMargin":"0","maintMargin":"0","updateTime":0},{"symbol":"BTCUSDT","positionSide":"BOTH","positionAmt":"1.000","unrealizedProfit":"0.00000000","isolatedMargin":"0.00000000","notional":"0","isolatedWallet":"0","initialMargin":"0","maintMargin":"0","updateTime":0}]}"#).unwrap();
//             let expected_response : models::AccountInformationV3Response = serde_json::from_value(resp_json.clone()).expect("should parse into models::AccountInformationV3Response");

//             let resp = client.account_information_v3(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn account_information_v3_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = AccountInformationV3Params::builder().recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"totalInitialMargin":"0.00000000","totalMaintMargin":"0.00000000","totalWalletBalance":"126.72469206","totalUnrealizedProfit":"0.00000000","totalMarginBalance":"126.72469206","totalPositionInitialMargin":"0.00000000","totalOpenOrderInitialMargin":"0.00000000","totalCrossWalletBalance":"126.72469206","totalCrossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"126.72469206","assets":[{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","updateTime":1625474304765},{"asset":"USDC","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"103.12345678","updateTime":1625474304765},{"asset":"USDT","walletBalance":"23.72469206","unrealizedProfit":"0.00000000","marginBalance":"23.72469206","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1625474304765},{"asset":"BUSD","walletBalance":"103.12345678","unrealizedProfit":"0.00000000","marginBalance":"103.12345678","maintMargin":"0.00000000","initialMargin":"0.00000000","positionInitialMargin":"0.00000000","openOrderInitialMargin":"0.00000000","crossWalletBalance":"103.12345678","crossUnPnl":"0.00000000","availableBalance":"126.72469206","maxWithdrawAmount":"103.12345678","marginAvailable":true,"updateTime":1625474304765}],"positions":[{"symbol":"BTCUSDT","positionSide":"BOTH","positionAmt":"1.000","unrealizedProfit":"0.00000000","isolatedMargin":"0.00000000","notional":"0","isolatedWallet":"0","initialMargin":"0","maintMargin":"0","updateTime":0},{"symbol":"BTCUSDT","positionSide":"BOTH","positionAmt":"1.000","unrealizedProfit":"0.00000000","isolatedMargin":"0.00000000","notional":"0","isolatedWallet":"0","initialMargin":"0","maintMargin":"0","updateTime":0}]}"#).unwrap();
//             let expected_response : models::AccountInformationV3Response = serde_json::from_value(resp_json.clone()).expect("should parse into models::AccountInformationV3Response");

//             let resp = client.account_information_v3(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn account_information_v3_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = AccountInformationV3Params::builder().build().unwrap();

//             match client.account_information_v3(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn futures_account_balance_v2_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = FuturesAccountBalanceV2Params::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"accountAlias":"SgsR","asset":"USDT","balance":"122607.35137903","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1617939110373}]"#).unwrap();
//             let expected_response : Vec<models::FuturesAccountBalanceV2ResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::FuturesAccountBalanceV2ResponseInner>");

//             let resp = client.futures_account_balance_v2(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn futures_account_balance_v2_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = FuturesAccountBalanceV2Params::builder().recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"accountAlias":"SgsR","asset":"USDT","balance":"122607.35137903","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1617939110373}]"#).unwrap();
//             let expected_response : Vec<models::FuturesAccountBalanceV2ResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::FuturesAccountBalanceV2ResponseInner>");

//             let resp = client.futures_account_balance_v2(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn futures_account_balance_v2_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = FuturesAccountBalanceV2Params::builder().build().unwrap();

//             match client.futures_account_balance_v2(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn futures_account_balance_v3_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = FuturesAccountBalanceV3Params::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"accountAlias":"SgsR","asset":"USDT","balance":"122607.35137903","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1617939110373}]"#).unwrap();
//             let expected_response : Vec<models::FuturesAccountBalanceV2ResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::FuturesAccountBalanceV2ResponseInner>");

//             let resp = client.futures_account_balance_v3(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn futures_account_balance_v3_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = FuturesAccountBalanceV3Params::builder().recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"accountAlias":"SgsR","asset":"USDT","balance":"122607.35137903","crossWalletBalance":"23.72469206","crossUnPnl":"0.00000000","availableBalance":"23.72469206","maxWithdrawAmount":"23.72469206","marginAvailable":true,"updateTime":1617939110373}]"#).unwrap();
//             let expected_response : Vec<models::FuturesAccountBalanceV2ResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::FuturesAccountBalanceV2ResponseInner>");

//             let resp = client.futures_account_balance_v3(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn futures_account_balance_v3_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = FuturesAccountBalanceV3Params::builder().build().unwrap();

//             match client.futures_account_balance_v3(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn futures_account_configuration_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = FuturesAccountConfigurationParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"feeTier":0,"canTrade":true,"canDeposit":true,"canWithdraw":true,"dualSidePosition":true,"updateTime":0,"multiAssetsMargin":false,"tradeGroupId":-1}"#).unwrap();
//             let expected_response : models::FuturesAccountConfigurationResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::FuturesAccountConfigurationResponse");

//             let resp = client.futures_account_configuration(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn futures_account_configuration_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = FuturesAccountConfigurationParams::builder().recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"feeTier":0,"canTrade":true,"canDeposit":true,"canWithdraw":true,"dualSidePosition":true,"updateTime":0,"multiAssetsMargin":false,"tradeGroupId":-1}"#).unwrap();
//             let expected_response : models::FuturesAccountConfigurationResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::FuturesAccountConfigurationResponse");

//             let resp = client.futures_account_configuration(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn futures_account_configuration_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = FuturesAccountConfigurationParams::builder().build().unwrap();

//             match client.futures_account_configuration(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn futures_trading_quantitative_rules_indicators_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = FuturesTradingQuantitativeRulesIndicatorsParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"indicators":{"BTCUSDT":[{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"UFR","value":0.05,"triggerValue":0.995},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"IFER","value":0.99,"triggerValue":0.99},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"GCR","value":0.99,"triggerValue":0.99},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"DR","value":0.99,"triggerValue":0.99}],"ETHUSDT":[{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"UFR","value":0.05,"triggerValue":0.995},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"IFER","value":0.99,"triggerValue":0.99},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"GCR","value":0.99,"triggerValue":0.99},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"DR","value":0.99,"triggerValue":0.99}],"ACCOUNT":[{"indicator":"TMV","value":10,"triggerValue":1,"plannedRecoverTime":1644919865000,"isLocked":true}]},"updateTime":1644913304748}"#).unwrap();
//             let expected_response : models::FuturesTradingQuantitativeRulesIndicatorsResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::FuturesTradingQuantitativeRulesIndicatorsResponse");

//             let resp = client.futures_trading_quantitative_rules_indicators(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn futures_trading_quantitative_rules_indicators_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = FuturesTradingQuantitativeRulesIndicatorsParams::builder().symbol("symbol_example".to_string()).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"indicators":{"BTCUSDT":[{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"UFR","value":0.05,"triggerValue":0.995},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"IFER","value":0.99,"triggerValue":0.99},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"GCR","value":0.99,"triggerValue":0.99},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"DR","value":0.99,"triggerValue":0.99}],"ETHUSDT":[{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"UFR","value":0.05,"triggerValue":0.995},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"IFER","value":0.99,"triggerValue":0.99},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"GCR","value":0.99,"triggerValue":0.99},{"isLocked":true,"plannedRecoverTime":1545741270000,"indicator":"DR","value":0.99,"triggerValue":0.99}],"ACCOUNT":[{"indicator":"TMV","value":10,"triggerValue":1,"plannedRecoverTime":1644919865000,"isLocked":true}]},"updateTime":1644913304748}"#).unwrap();
//             let expected_response : models::FuturesTradingQuantitativeRulesIndicatorsResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::FuturesTradingQuantitativeRulesIndicatorsResponse");

//             let resp = client.futures_trading_quantitative_rules_indicators(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn futures_trading_quantitative_rules_indicators_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = FuturesTradingQuantitativeRulesIndicatorsParams::builder().build().unwrap();

//             match client.futures_trading_quantitative_rules_indicators(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_bnb_burn_status_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetBnbBurnStatusParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"feeBurn":true}"#).unwrap();
//             let expected_response: models::GetBnbBurnStatusResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::GetBnbBurnStatusResponse");

//             let resp = client.get_bnb_burn_status(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_bnb_burn_status_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetBnbBurnStatusParams::builder().recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"feeBurn":true}"#).unwrap();
//             let expected_response: models::GetBnbBurnStatusResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::GetBnbBurnStatusResponse");

//             let resp = client.get_bnb_burn_status(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_bnb_burn_status_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = GetBnbBurnStatusParams::builder().build().unwrap();

//             match client.get_bnb_burn_status(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_current_multi_assets_mode_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetCurrentMultiAssetsModeParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"multiAssetsMargin":true}"#).unwrap();
//             let expected_response: models::GetCurrentMultiAssetsModeResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetCurrentMultiAssetsModeResponse");

//             let resp = client.get_current_multi_assets_mode(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_current_multi_assets_mode_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetCurrentMultiAssetsModeParams::builder().recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"multiAssetsMargin":true}"#).unwrap();
//             let expected_response: models::GetCurrentMultiAssetsModeResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetCurrentMultiAssetsModeResponse");

//             let resp = client.get_current_multi_assets_mode(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_current_multi_assets_mode_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = GetCurrentMultiAssetsModeParams::builder().build().unwrap();

//             match client.get_current_multi_assets_mode(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_current_position_mode_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetCurrentPositionModeParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"dualSidePosition":true}"#).unwrap();
//             let expected_response: models::GetCurrentPositionModeResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::GetCurrentPositionModeResponse");

//             let resp = client.get_current_position_mode(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_current_position_mode_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetCurrentPositionModeParams::builder().recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"dualSidePosition":true}"#).unwrap();
//             let expected_response: models::GetCurrentPositionModeResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::GetCurrentPositionModeResponse");

//             let resp = client.get_current_position_mode(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_current_position_mode_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = GetCurrentPositionModeParams::builder().build().unwrap();

//             match client.get_current_position_mode(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_download_id_for_futures_order_history_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetDownloadIdForFuturesOrderHistoryParams::builder(1623319461670, 1641782889000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"avgCostTimestampOfLast30d":7241837,"downloadId":"546975389218332672"}"#)
//                     .unwrap();
//             let expected_response: models::GetDownloadIdForFuturesOrderHistoryResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetDownloadIdForFuturesOrderHistoryResponse");

//             let resp = client
//                 .get_download_id_for_futures_order_history(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_download_id_for_futures_order_history_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetDownloadIdForFuturesOrderHistoryParams::builder(1623319461670, 1641782889000)
//                 .recv_window(5000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"avgCostTimestampOfLast30d":7241837,"downloadId":"546975389218332672"}"#)
//                     .unwrap();
//             let expected_response: models::GetDownloadIdForFuturesOrderHistoryResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetDownloadIdForFuturesOrderHistoryResponse");

//             let resp = client
//                 .get_download_id_for_futures_order_history(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_download_id_for_futures_order_history_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = GetDownloadIdForFuturesOrderHistoryParams::builder(1623319461670, 1641782889000)
//                 .build()
//                 .unwrap();

//             match client.get_download_id_for_futures_order_history(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_download_id_for_futures_trade_history_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetDownloadIdForFuturesTradeHistoryParams::builder(1623319461670, 1641782889000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"avgCostTimestampOfLast30d":7241837,"downloadId":"546975389218332672"}"#)
//                     .unwrap();
//             let expected_response: models::GetDownloadIdForFuturesTradeHistoryResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetDownloadIdForFuturesTradeHistoryResponse");

//             let resp = client
//                 .get_download_id_for_futures_trade_history(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_download_id_for_futures_trade_history_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetDownloadIdForFuturesTradeHistoryParams::builder(1623319461670, 1641782889000)
//                 .recv_window(5000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"avgCostTimestampOfLast30d":7241837,"downloadId":"546975389218332672"}"#)
//                     .unwrap();
//             let expected_response: models::GetDownloadIdForFuturesTradeHistoryResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetDownloadIdForFuturesTradeHistoryResponse");

//             let resp = client
//                 .get_download_id_for_futures_trade_history(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_download_id_for_futures_trade_history_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = GetDownloadIdForFuturesTradeHistoryParams::builder(1623319461670, 1641782889000)
//                 .build()
//                 .unwrap();

//             match client.get_download_id_for_futures_trade_history(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_download_id_for_futures_transaction_history_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetDownloadIdForFuturesTransactionHistoryParams::builder(1623319461670, 1641782889000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"avgCostTimestampOfLast30d":7241837,"downloadId":"546975389218332672"}"#)
//                     .unwrap();
//             let expected_response: models::GetDownloadIdForFuturesTransactionHistoryResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetDownloadIdForFuturesTransactionHistoryResponse");

//             let resp = client
//                 .get_download_id_for_futures_transaction_history(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_download_id_for_futures_transaction_history_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetDownloadIdForFuturesTransactionHistoryParams::builder(1623319461670, 1641782889000)
//                 .recv_window(5000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"avgCostTimestampOfLast30d":7241837,"downloadId":"546975389218332672"}"#)
//                     .unwrap();
//             let expected_response: models::GetDownloadIdForFuturesTransactionHistoryResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::GetDownloadIdForFuturesTransactionHistoryResponse");

//             let resp = client
//                 .get_download_id_for_futures_transaction_history(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_download_id_for_futures_transaction_history_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = GetDownloadIdForFuturesTransactionHistoryParams::builder(1623319461670, 1641782889000)
//                 .build()
//                 .unwrap();

//             match client.get_download_id_for_futures_transaction_history(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_futures_order_history_download_link_by_id_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetFuturesOrderHistoryDownloadLinkByIdParams::builder("1".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"downloadId":"545923594199212032","status":"processing","url":"","notified":false,"expirationTimestamp":-1,"isExpired":null}"#).unwrap();
//             let expected_response : models::GetFuturesOrderHistoryDownloadLinkByIdResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::GetFuturesOrderHistoryDownloadLinkByIdResponse");

//             let resp = client.get_futures_order_history_download_link_by_id(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_futures_order_history_download_link_by_id_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetFuturesOrderHistoryDownloadLinkByIdParams::builder("1".to_string(),).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"downloadId":"545923594199212032","status":"processing","url":"","notified":false,"expirationTimestamp":-1,"isExpired":null}"#).unwrap();
//             let expected_response : models::GetFuturesOrderHistoryDownloadLinkByIdResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::GetFuturesOrderHistoryDownloadLinkByIdResponse");

//             let resp = client.get_futures_order_history_download_link_by_id(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_futures_order_history_download_link_by_id_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = GetFuturesOrderHistoryDownloadLinkByIdParams::builder("1".to_string())
//                 .build()
//                 .unwrap();

//             match client.get_futures_order_history_download_link_by_id(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_futures_trade_download_link_by_id_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetFuturesTradeDownloadLinkByIdParams::builder("1".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"downloadId":"545923594199212032","status":"processing","url":"","notified":false,"expirationTimestamp":-1,"isExpired":null}"#).unwrap();
//             let expected_response : models::GetFuturesTradeDownloadLinkByIdResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::GetFuturesTradeDownloadLinkByIdResponse");

//             let resp = client.get_futures_trade_download_link_by_id(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_futures_trade_download_link_by_id_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetFuturesTradeDownloadLinkByIdParams::builder("1".to_string(),).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"downloadId":"545923594199212032","status":"processing","url":"","notified":false,"expirationTimestamp":-1,"isExpired":null}"#).unwrap();
//             let expected_response : models::GetFuturesTradeDownloadLinkByIdResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::GetFuturesTradeDownloadLinkByIdResponse");

//             let resp = client.get_futures_trade_download_link_by_id(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_futures_trade_download_link_by_id_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = GetFuturesTradeDownloadLinkByIdParams::builder("1".to_string()).build().unwrap();

//             match client.get_futures_trade_download_link_by_id(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_futures_transaction_history_download_link_by_id_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetFuturesTransactionHistoryDownloadLinkByIdParams::builder("1".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"downloadId":"545923594199212032","status":"processing","url":"","notified":false,"expirationTimestamp":-1,"isExpired":null}"#).unwrap();
//             let expected_response : models::GetFuturesTransactionHistoryDownloadLinkByIdResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::GetFuturesTransactionHistoryDownloadLinkByIdResponse");

//             let resp = client.get_futures_transaction_history_download_link_by_id(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_futures_transaction_history_download_link_by_id_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetFuturesTransactionHistoryDownloadLinkByIdParams::builder("1".to_string(),).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"downloadId":"545923594199212032","status":"processing","url":"","notified":false,"expirationTimestamp":-1,"isExpired":null}"#).unwrap();
//             let expected_response : models::GetFuturesTransactionHistoryDownloadLinkByIdResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::GetFuturesTransactionHistoryDownloadLinkByIdResponse");

//             let resp = client.get_futures_transaction_history_download_link_by_id(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_futures_transaction_history_download_link_by_id_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = GetFuturesTransactionHistoryDownloadLinkByIdParams::builder("1".to_string())
//                 .build()
//                 .unwrap();

//             match client.get_futures_transaction_history_download_link_by_id(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_income_history_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetIncomeHistoryParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"","incomeType":"TRANSFER","income":"-0.37500000","asset":"USDT","info":"TRANSFER","time":1570608000000,"tranId":9689322392,"tradeId":""},{"symbol":"BTCUSDT","incomeType":"COMMISSION","income":"-0.01000000","asset":"USDT","info":"COMMISSION","time":1570636800000,"tranId":9689322392,"tradeId":"2059192"}]"#).unwrap();
//             let expected_response : Vec<models::GetIncomeHistoryResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::GetIncomeHistoryResponseInner>");

//             let resp = client.get_income_history(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_income_history_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = GetIncomeHistoryParams::builder().symbol("symbol_example".to_string()).income_type("income_type_example".to_string()).start_time(1623319461670).end_time(1641782889000).page(789).limit(100).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"","incomeType":"TRANSFER","income":"-0.37500000","asset":"USDT","info":"TRANSFER","time":1570608000000,"tranId":9689322392,"tradeId":""},{"symbol":"BTCUSDT","incomeType":"COMMISSION","income":"-0.01000000","asset":"USDT","info":"COMMISSION","time":1570636800000,"tranId":9689322392,"tradeId":"2059192"}]"#).unwrap();
//             let expected_response : Vec<models::GetIncomeHistoryResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::GetIncomeHistoryResponseInner>");

//             let resp = client.get_income_history(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_income_history_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = GetIncomeHistoryParams::builder().build().unwrap();

//             match client.get_income_history(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn notional_and_leverage_brackets_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = NotionalAndLeverageBracketsParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"ETHUSDT","notionalCoef":1.5,"brackets":[{"bracket":1,"initialLeverage":75,"notionalCap":10000,"notionalFloor":0,"maintMarginRatio":0.0065,"cum":0}]}]"#).unwrap();
//             let expected_response : models::NotionalAndLeverageBracketsResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::NotionalAndLeverageBracketsResponse");

//             let resp = client.notional_and_leverage_brackets(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn notional_and_leverage_brackets_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = NotionalAndLeverageBracketsParams::builder().symbol("symbol_example".to_string()).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"ETHUSDT","notionalCoef":1.5,"brackets":[{"bracket":1,"initialLeverage":75,"notionalCap":10000,"notionalFloor":0,"maintMarginRatio":0.0065,"cum":0}]}]"#).unwrap();
//             let expected_response : models::NotionalAndLeverageBracketsResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::NotionalAndLeverageBracketsResponse");

//             let resp = client.notional_and_leverage_brackets(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn notional_and_leverage_brackets_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = NotionalAndLeverageBracketsParams::builder().build().unwrap();

//             match client.notional_and_leverage_brackets(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn query_user_rate_limit_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = QueryUserRateLimitParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"rateLimitType":"ORDERS","interval":"SECOND","intervalNum":10,"limit":10000},{"rateLimitType":"ORDERS","interval":"MINUTE","intervalNum":1,"limit":20000}]"#).unwrap();
//             let expected_response : Vec<models::QueryUserRateLimitResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::QueryUserRateLimitResponseInner>");

//             let resp = client.query_user_rate_limit(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn query_user_rate_limit_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = QueryUserRateLimitParams::builder().recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"rateLimitType":"ORDERS","interval":"SECOND","intervalNum":10,"limit":10000},{"rateLimitType":"ORDERS","interval":"MINUTE","intervalNum":1,"limit":20000}]"#).unwrap();
//             let expected_response : Vec<models::QueryUserRateLimitResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::QueryUserRateLimitResponseInner>");

//             let resp = client.query_user_rate_limit(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn query_user_rate_limit_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = QueryUserRateLimitParams::builder().build().unwrap();

//             match client.query_user_rate_limit(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn symbol_configuration_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = SymbolConfigurationParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","marginType":"CROSSED","isAutoAddMargin":"false","leverage":21,"maxNotionalValue":"1000000"}]"#).unwrap();
//             let expected_response : Vec<models::SymbolConfigurationResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::SymbolConfigurationResponseInner>");

//             let resp = client.symbol_configuration(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn symbol_configuration_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = SymbolConfigurationParams::builder().symbol("symbol_example".to_string()).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","marginType":"CROSSED","isAutoAddMargin":"false","leverage":21,"maxNotionalValue":"1000000"}]"#).unwrap();
//             let expected_response : Vec<models::SymbolConfigurationResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::SymbolConfigurationResponseInner>");

//             let resp = client.symbol_configuration(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn symbol_configuration_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = SymbolConfigurationParams::builder().build().unwrap();

//             match client.symbol_configuration(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn toggle_bnb_burn_on_futures_trade_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = ToggleBnbBurnOnFuturesTradeParams::builder("fee_burn_example".to_string())
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"code":200,"msg":"success"}"#).unwrap();
//             let expected_response: models::ToggleBnbBurnOnFuturesTradeResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::ToggleBnbBurnOnFuturesTradeResponse");

//             let resp = client
//                 .toggle_bnb_burn_on_futures_trade(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn toggle_bnb_burn_on_futures_trade_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = ToggleBnbBurnOnFuturesTradeParams::builder("fee_burn_example".to_string())
//                 .recv_window(5000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"code":200,"msg":"success"}"#).unwrap();
//             let expected_response: models::ToggleBnbBurnOnFuturesTradeResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::ToggleBnbBurnOnFuturesTradeResponse");

//             let resp = client
//                 .toggle_bnb_burn_on_futures_trade(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn toggle_bnb_burn_on_futures_trade_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = ToggleBnbBurnOnFuturesTradeParams::builder("fee_burn_example".to_string())
//                 .build()
//                 .unwrap();

//             match client.toggle_bnb_burn_on_futures_trade(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn user_commission_rate_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = UserCommissionRateParams::builder("symbol_example".to_string()).build().unwrap();

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"symbol":"BTCUSDT","makerCommissionRate":"0.0002","takerCommissionRate":"0.0004"}"#,
//             )
//             .unwrap();
//             let expected_response: models::UserCommissionRateResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::UserCommissionRateResponse");

//             let resp = client.user_commission_rate(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn user_commission_rate_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: false };

//             let params = UserCommissionRateParams::builder("symbol_example".to_string())
//                 .recv_window(5000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"symbol":"BTCUSDT","makerCommissionRate":"0.0002","takerCommissionRate":"0.0004"}"#,
//             )
//             .unwrap();
//             let expected_response: models::UserCommissionRateResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::UserCommissionRateResponse");

//             let resp = client.user_commission_rate(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn user_commission_rate_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockAccountApiClient { force_error: true };

//             let params = UserCommissionRateParams::builder("symbol_example".to_string()).build().unwrap();

//             match client.user_commission_rate(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }
// }
