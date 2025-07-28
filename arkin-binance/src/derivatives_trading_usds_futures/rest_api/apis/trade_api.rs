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
pub trait TradeApi: Send + Sync {
    async fn account_trade_list(
        &self,
        params: AccountTradeListParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::AccountTradeListResponseInner>>>;
    async fn all_orders(
        &self,
        params: AllOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::AllOrdersResponseInner>>>;
    async fn auto_cancel_all_open_orders(
        &self,
        params: AutoCancelAllOpenOrdersParams,
    ) -> anyhow::Result<RestApiResponse<models::AutoCancelAllOpenOrdersResponse>>;
    async fn cancel_all_open_orders(
        &self,
        params: CancelAllOpenOrdersParams,
    ) -> anyhow::Result<RestApiResponse<models::CancelAllOpenOrdersResponse>>;
    async fn cancel_multiple_orders(
        &self,
        params: CancelMultipleOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::CancelMultipleOrdersResponseInner>>>;
    async fn cancel_order(
        &self,
        params: CancelOrderParams,
    ) -> anyhow::Result<RestApiResponse<models::CancelOrderResponse>>;
    async fn change_initial_leverage(
        &self,
        params: ChangeInitialLeverageParams,
    ) -> anyhow::Result<RestApiResponse<models::ChangeInitialLeverageResponse>>;
    async fn change_margin_type(
        &self,
        params: ChangeMarginTypeParams,
    ) -> anyhow::Result<RestApiResponse<models::ChangeMarginTypeResponse>>;
    async fn change_multi_assets_mode(
        &self,
        params: ChangeMultiAssetsModeParams,
    ) -> anyhow::Result<RestApiResponse<models::ChangeMultiAssetsModeResponse>>;
    async fn change_position_mode(
        &self,
        params: ChangePositionModeParams,
    ) -> anyhow::Result<RestApiResponse<models::ChangePositionModeResponse>>;
    async fn current_all_open_orders(
        &self,
        params: CurrentAllOpenOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::AllOrdersResponseInner>>>;
    async fn get_order_modify_history(
        &self,
        params: GetOrderModifyHistoryParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetOrderModifyHistoryResponseInner>>>;
    async fn get_position_margin_change_history(
        &self,
        params: GetPositionMarginChangeHistoryParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetPositionMarginChangeHistoryResponseInner>>>;
    async fn modify_isolated_position_margin(
        &self,
        params: ModifyIsolatedPositionMarginParams,
    ) -> anyhow::Result<RestApiResponse<models::ModifyIsolatedPositionMarginResponse>>;
    async fn modify_multiple_orders(
        &self,
        params: ModifyMultipleOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::ModifyMultipleOrdersResponseInner>>>;
    async fn modify_order(
        &self,
        params: ModifyOrderParams,
    ) -> anyhow::Result<RestApiResponse<models::ModifyOrderResponse>>;
    async fn new_order(&self, params: NewOrderParams) -> anyhow::Result<RestApiResponse<models::NewOrderResponse>>;
    async fn place_multiple_orders(
        &self,
        params: PlaceMultipleOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::PlaceMultipleOrdersResponseInner>>>;
    async fn position_adl_quantile_estimation(
        &self,
        params: PositionAdlQuantileEstimationParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::PositionAdlQuantileEstimationResponseInner>>>;
    async fn position_information_v2(
        &self,
        params: PositionInformationV2Params,
    ) -> anyhow::Result<RestApiResponse<Vec<models::PositionInformationV2ResponseInner>>>;
    async fn position_information_v3(
        &self,
        params: PositionInformationV3Params,
    ) -> anyhow::Result<RestApiResponse<Vec<models::PositionInformationV3ResponseInner>>>;
    async fn query_current_open_order(
        &self,
        params: QueryCurrentOpenOrderParams,
    ) -> anyhow::Result<RestApiResponse<models::QueryCurrentOpenOrderResponse>>;
    async fn query_order(
        &self,
        params: QueryOrderParams,
    ) -> anyhow::Result<RestApiResponse<models::QueryOrderResponse>>;
    async fn test_order(&self, params: TestOrderParams) -> anyhow::Result<RestApiResponse<models::TestOrderResponse>>;
    async fn users_force_orders(
        &self,
        params: UsersForceOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::UsersForceOrdersResponseInner>>>;
}

#[derive(Debug, Clone)]
pub struct TradeApiClient {
    configuration: ConfigurationRestApi,
}

impl TradeApiClient {
    pub fn new(configuration: ConfigurationRestApi) -> Self {
        Self { configuration }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeMarginTypeMarginTypeEnum {
    #[serde(rename = "ISOLATED")]
    Isolated,
    #[serde(rename = "CROSSED")]
    Crossed,
}

impl ChangeMarginTypeMarginTypeEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Isolated => "ISOLATED",
            Self::Crossed => "CROSSED",
        }
    }
}

impl std::str::FromStr for ChangeMarginTypeMarginTypeEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ISOLATED" => Ok(Self::Isolated),
            "CROSSED" => Ok(Self::Crossed),
            other => Err(format!("invalid ChangeMarginTypeMarginTypeEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModifyIsolatedPositionMarginPositionSideEnum {
    #[serde(rename = "BOTH")]
    Both,
    #[serde(rename = "LONG")]
    Long,
    #[serde(rename = "SHORT")]
    Short,
}

impl ModifyIsolatedPositionMarginPositionSideEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Both => "BOTH",
            Self::Long => "LONG",
            Self::Short => "SHORT",
        }
    }
}

impl std::str::FromStr for ModifyIsolatedPositionMarginPositionSideEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BOTH" => Ok(Self::Both),
            "LONG" => Ok(Self::Long),
            "SHORT" => Ok(Self::Short),
            other => Err(format!("invalid ModifyIsolatedPositionMarginPositionSideEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModifyOrderSideEnum {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

impl ModifyOrderSideEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }
}

impl std::str::FromStr for ModifyOrderSideEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BUY" => Ok(Self::Buy),
            "SELL" => Ok(Self::Sell),
            other => Err(format!("invalid ModifyOrderSideEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModifyOrderPriceMatchEnum {
    #[serde(rename = "NONE")]
    None,
    #[serde(rename = "OPPONENT")]
    Opponent,
    #[serde(rename = "OPPONENT_5")]
    Opponent5,
    #[serde(rename = "OPPONENT_10")]
    Opponent10,
    #[serde(rename = "OPPONENT_20")]
    Opponent20,
    #[serde(rename = "QUEUE")]
    Queue,
    #[serde(rename = "QUEUE_5")]
    Queue5,
    #[serde(rename = "QUEUE_10")]
    Queue10,
    #[serde(rename = "QUEUE_20")]
    Queue20,
}

impl ModifyOrderPriceMatchEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Opponent => "OPPONENT",
            Self::Opponent5 => "OPPONENT_5",
            Self::Opponent10 => "OPPONENT_10",
            Self::Opponent20 => "OPPONENT_20",
            Self::Queue => "QUEUE",
            Self::Queue5 => "QUEUE_5",
            Self::Queue10 => "QUEUE_10",
            Self::Queue20 => "QUEUE_20",
        }
    }
}

impl std::str::FromStr for ModifyOrderPriceMatchEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NONE" => Ok(Self::None),
            "OPPONENT" => Ok(Self::Opponent),
            "OPPONENT_5" => Ok(Self::Opponent5),
            "OPPONENT_10" => Ok(Self::Opponent10),
            "OPPONENT_20" => Ok(Self::Opponent20),
            "QUEUE" => Ok(Self::Queue),
            "QUEUE_5" => Ok(Self::Queue5),
            "QUEUE_10" => Ok(Self::Queue10),
            "QUEUE_20" => Ok(Self::Queue20),
            other => Err(format!("invalid ModifyOrderPriceMatchEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewOrderSideEnum {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

impl NewOrderSideEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }
}

impl std::str::FromStr for NewOrderSideEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BUY" => Ok(Self::Buy),
            "SELL" => Ok(Self::Sell),
            other => Err(format!("invalid NewOrderSideEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewOrderPositionSideEnum {
    #[serde(rename = "BOTH")]
    Both,
    #[serde(rename = "LONG")]
    Long,
    #[serde(rename = "SHORT")]
    Short,
}

impl NewOrderPositionSideEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Both => "BOTH",
            Self::Long => "LONG",
            Self::Short => "SHORT",
        }
    }
}

impl std::str::FromStr for NewOrderPositionSideEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BOTH" => Ok(Self::Both),
            "LONG" => Ok(Self::Long),
            "SHORT" => Ok(Self::Short),
            other => Err(format!("invalid NewOrderPositionSideEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewOrderTimeInForceEnum {
    #[serde(rename = "GTC")]
    Gtc,
    #[serde(rename = "IOC")]
    Ioc,
    #[serde(rename = "FOK")]
    Fok,
    #[serde(rename = "GTX")]
    Gtx,
    #[serde(rename = "GTD")]
    Gtd,
}

impl NewOrderTimeInForceEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gtc => "GTC",
            Self::Ioc => "IOC",
            Self::Fok => "FOK",
            Self::Gtx => "GTX",
            Self::Gtd => "GTD",
        }
    }
}

impl std::str::FromStr for NewOrderTimeInForceEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GTC" => Ok(Self::Gtc),
            "IOC" => Ok(Self::Ioc),
            "FOK" => Ok(Self::Fok),
            "GTX" => Ok(Self::Gtx),
            "GTD" => Ok(Self::Gtd),
            other => Err(format!("invalid NewOrderTimeInForceEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewOrderWorkingTypeEnum {
    #[serde(rename = "MARK_PRICE")]
    MarkPrice,
    #[serde(rename = "CONTRACT_PRICE")]
    ContractPrice,
}

impl NewOrderWorkingTypeEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MarkPrice => "MARK_PRICE",
            Self::ContractPrice => "CONTRACT_PRICE",
        }
    }
}

impl std::str::FromStr for NewOrderWorkingTypeEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MARK_PRICE" => Ok(Self::MarkPrice),
            "CONTRACT_PRICE" => Ok(Self::ContractPrice),
            other => Err(format!("invalid NewOrderWorkingTypeEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewOrderNewOrderRespTypeEnum {
    #[serde(rename = "ACK")]
    Ack,
    #[serde(rename = "RESULT")]
    Result,
}

impl NewOrderNewOrderRespTypeEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ack => "ACK",
            Self::Result => "RESULT",
        }
    }
}

impl std::str::FromStr for NewOrderNewOrderRespTypeEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACK" => Ok(Self::Ack),
            "RESULT" => Ok(Self::Result),
            other => Err(format!("invalid NewOrderNewOrderRespTypeEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewOrderPriceMatchEnum {
    #[serde(rename = "NONE")]
    None,
    #[serde(rename = "OPPONENT")]
    Opponent,
    #[serde(rename = "OPPONENT_5")]
    Opponent5,
    #[serde(rename = "OPPONENT_10")]
    Opponent10,
    #[serde(rename = "OPPONENT_20")]
    Opponent20,
    #[serde(rename = "QUEUE")]
    Queue,
    #[serde(rename = "QUEUE_5")]
    Queue5,
    #[serde(rename = "QUEUE_10")]
    Queue10,
    #[serde(rename = "QUEUE_20")]
    Queue20,
}

impl NewOrderPriceMatchEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Opponent => "OPPONENT",
            Self::Opponent5 => "OPPONENT_5",
            Self::Opponent10 => "OPPONENT_10",
            Self::Opponent20 => "OPPONENT_20",
            Self::Queue => "QUEUE",
            Self::Queue5 => "QUEUE_5",
            Self::Queue10 => "QUEUE_10",
            Self::Queue20 => "QUEUE_20",
        }
    }
}

impl std::str::FromStr for NewOrderPriceMatchEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NONE" => Ok(Self::None),
            "OPPONENT" => Ok(Self::Opponent),
            "OPPONENT_5" => Ok(Self::Opponent5),
            "OPPONENT_10" => Ok(Self::Opponent10),
            "OPPONENT_20" => Ok(Self::Opponent20),
            "QUEUE" => Ok(Self::Queue),
            "QUEUE_5" => Ok(Self::Queue5),
            "QUEUE_10" => Ok(Self::Queue10),
            "QUEUE_20" => Ok(Self::Queue20),
            other => Err(format!("invalid NewOrderPriceMatchEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NewOrderSelfTradePreventionModeEnum {
    #[serde(rename = "EXPIRE_TAKER")]
    ExpireTaker,
    #[serde(rename = "EXPIRE_BOTH")]
    ExpireBoth,
    #[serde(rename = "EXPIRE_MAKER")]
    ExpireMaker,
}

impl NewOrderSelfTradePreventionModeEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExpireTaker => "EXPIRE_TAKER",
            Self::ExpireBoth => "EXPIRE_BOTH",
            Self::ExpireMaker => "EXPIRE_MAKER",
        }
    }
}

impl std::str::FromStr for NewOrderSelfTradePreventionModeEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EXPIRE_TAKER" => Ok(Self::ExpireTaker),
            "EXPIRE_BOTH" => Ok(Self::ExpireBoth),
            "EXPIRE_MAKER" => Ok(Self::ExpireMaker),
            other => Err(format!("invalid NewOrderSelfTradePreventionModeEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestOrderSideEnum {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

impl TestOrderSideEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Buy => "BUY",
            Self::Sell => "SELL",
        }
    }
}

impl std::str::FromStr for TestOrderSideEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BUY" => Ok(Self::Buy),
            "SELL" => Ok(Self::Sell),
            other => Err(format!("invalid TestOrderSideEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestOrderPositionSideEnum {
    #[serde(rename = "BOTH")]
    Both,
    #[serde(rename = "LONG")]
    Long,
    #[serde(rename = "SHORT")]
    Short,
}

impl TestOrderPositionSideEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Both => "BOTH",
            Self::Long => "LONG",
            Self::Short => "SHORT",
        }
    }
}

impl std::str::FromStr for TestOrderPositionSideEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BOTH" => Ok(Self::Both),
            "LONG" => Ok(Self::Long),
            "SHORT" => Ok(Self::Short),
            other => Err(format!("invalid TestOrderPositionSideEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestOrderTimeInForceEnum {
    #[serde(rename = "GTC")]
    Gtc,
    #[serde(rename = "IOC")]
    Ioc,
    #[serde(rename = "FOK")]
    Fok,
    #[serde(rename = "GTX")]
    Gtx,
    #[serde(rename = "GTD")]
    Gtd,
}

impl TestOrderTimeInForceEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Gtc => "GTC",
            Self::Ioc => "IOC",
            Self::Fok => "FOK",
            Self::Gtx => "GTX",
            Self::Gtd => "GTD",
        }
    }
}

impl std::str::FromStr for TestOrderTimeInForceEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GTC" => Ok(Self::Gtc),
            "IOC" => Ok(Self::Ioc),
            "FOK" => Ok(Self::Fok),
            "GTX" => Ok(Self::Gtx),
            "GTD" => Ok(Self::Gtd),
            other => Err(format!("invalid TestOrderTimeInForceEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestOrderWorkingTypeEnum {
    #[serde(rename = "MARK_PRICE")]
    MarkPrice,
    #[serde(rename = "CONTRACT_PRICE")]
    ContractPrice,
}

impl TestOrderWorkingTypeEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MarkPrice => "MARK_PRICE",
            Self::ContractPrice => "CONTRACT_PRICE",
        }
    }
}

impl std::str::FromStr for TestOrderWorkingTypeEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "MARK_PRICE" => Ok(Self::MarkPrice),
            "CONTRACT_PRICE" => Ok(Self::ContractPrice),
            other => Err(format!("invalid TestOrderWorkingTypeEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestOrderNewOrderRespTypeEnum {
    #[serde(rename = "ACK")]
    Ack,
    #[serde(rename = "RESULT")]
    Result,
}

impl TestOrderNewOrderRespTypeEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ack => "ACK",
            Self::Result => "RESULT",
        }
    }
}

impl std::str::FromStr for TestOrderNewOrderRespTypeEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACK" => Ok(Self::Ack),
            "RESULT" => Ok(Self::Result),
            other => Err(format!("invalid TestOrderNewOrderRespTypeEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestOrderPriceMatchEnum {
    #[serde(rename = "NONE")]
    None,
    #[serde(rename = "OPPONENT")]
    Opponent,
    #[serde(rename = "OPPONENT_5")]
    Opponent5,
    #[serde(rename = "OPPONENT_10")]
    Opponent10,
    #[serde(rename = "OPPONENT_20")]
    Opponent20,
    #[serde(rename = "QUEUE")]
    Queue,
    #[serde(rename = "QUEUE_5")]
    Queue5,
    #[serde(rename = "QUEUE_10")]
    Queue10,
    #[serde(rename = "QUEUE_20")]
    Queue20,
}

impl TestOrderPriceMatchEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Opponent => "OPPONENT",
            Self::Opponent5 => "OPPONENT_5",
            Self::Opponent10 => "OPPONENT_10",
            Self::Opponent20 => "OPPONENT_20",
            Self::Queue => "QUEUE",
            Self::Queue5 => "QUEUE_5",
            Self::Queue10 => "QUEUE_10",
            Self::Queue20 => "QUEUE_20",
        }
    }
}

impl std::str::FromStr for TestOrderPriceMatchEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NONE" => Ok(Self::None),
            "OPPONENT" => Ok(Self::Opponent),
            "OPPONENT_5" => Ok(Self::Opponent5),
            "OPPONENT_10" => Ok(Self::Opponent10),
            "OPPONENT_20" => Ok(Self::Opponent20),
            "QUEUE" => Ok(Self::Queue),
            "QUEUE_5" => Ok(Self::Queue5),
            "QUEUE_10" => Ok(Self::Queue10),
            "QUEUE_20" => Ok(Self::Queue20),
            other => Err(format!("invalid TestOrderPriceMatchEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestOrderSelfTradePreventionModeEnum {
    #[serde(rename = "EXPIRE_TAKER")]
    ExpireTaker,
    #[serde(rename = "EXPIRE_BOTH")]
    ExpireBoth,
    #[serde(rename = "EXPIRE_MAKER")]
    ExpireMaker,
}

impl TestOrderSelfTradePreventionModeEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ExpireTaker => "EXPIRE_TAKER",
            Self::ExpireBoth => "EXPIRE_BOTH",
            Self::ExpireMaker => "EXPIRE_MAKER",
        }
    }
}

impl std::str::FromStr for TestOrderSelfTradePreventionModeEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "EXPIRE_TAKER" => Ok(Self::ExpireTaker),
            "EXPIRE_BOTH" => Ok(Self::ExpireBoth),
            "EXPIRE_MAKER" => Ok(Self::ExpireMaker),
            other => Err(format!("invalid TestOrderSelfTradePreventionModeEnum: {}", other).into()),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsersForceOrdersAutoCloseTypeEnum {
    #[serde(rename = "LIQUIDATION")]
    Liquidation,
    #[serde(rename = "ADL")]
    Adl,
}

impl UsersForceOrdersAutoCloseTypeEnum {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Liquidation => "LIQUIDATION",
            Self::Adl => "ADL",
        }
    }
}

impl std::str::FromStr for UsersForceOrdersAutoCloseTypeEnum {
    type Err = Box<dyn std::error::Error + Send + Sync>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LIQUIDATION" => Ok(Self::Liquidation),
            "ADL" => Ok(Self::Adl),
            other => Err(format!("invalid UsersForceOrdersAutoCloseTypeEnum: {}", other).into()),
        }
    }
}

/// Request parameters for the [`account_trade_list`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`account_trade_list`](#method.account_trade_list).
#[derive(Clone, Debug, TypedBuilder)]
pub struct AccountTradeListParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    ///
    /// The `order_id` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub order_id: Option<i64>,
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
    /// ID to get aggregate trades from INCLUSIVE.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub from_id: Option<i64>,
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

/// Request parameters for the [`all_orders`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`all_orders`](#method.all_orders).
#[derive(Clone, Debug, TypedBuilder)]
pub struct AllOrdersParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    ///
    /// The `order_id` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub order_id: Option<i64>,
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

/// Request parameters for the [`auto_cancel_all_open_orders`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`auto_cancel_all_open_orders`](#method.auto_cancel_all_open_orders).
#[derive(Clone, Debug, TypedBuilder)]
pub struct AutoCancelAllOpenOrdersParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// countdown time, 1000 for 1 second. 0 to cancel the timer
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub countdown_time: i64,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`cancel_all_open_orders`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`cancel_all_open_orders`](#method.cancel_all_open_orders).
#[derive(Clone, Debug, TypedBuilder)]
pub struct CancelAllOpenOrdersParams {
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

/// Request parameters for the [`cancel_multiple_orders`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`cancel_multiple_orders`](#method.cancel_multiple_orders).
#[derive(Clone, Debug, TypedBuilder)]
pub struct CancelMultipleOrdersParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// max length 10 <br /> e.g. [1234567,2345678]
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub order_id_list: Option<Vec<i64>>,
    /// max length 10<br /> e.g. ["`my_id_1","my_id_2`"], encode the double quotes. No space after comma.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub orig_client_order_id_list: Option<Vec<String>>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`cancel_order`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`cancel_order`](#method.cancel_order).
#[derive(Clone, Debug, TypedBuilder)]
pub struct CancelOrderParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    ///
    /// The `order_id` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub order_id: Option<i64>,
    ///
    /// The `orig_client_order_id` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub orig_client_order_id: Option<String>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`change_initial_leverage`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`change_initial_leverage`](#method.change_initial_leverage).
#[derive(Clone, Debug, TypedBuilder)]
pub struct ChangeInitialLeverageParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// target initial leverage: int from 1 to 125
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub leverage: i64,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`change_margin_type`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`change_margin_type`](#method.change_margin_type).
#[derive(Clone, Debug, TypedBuilder)]
pub struct ChangeMarginTypeParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// ISOLATED, CROSSED
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub margin_type: ChangeMarginTypeMarginTypeEnum,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`change_multi_assets_mode`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`change_multi_assets_mode`](#method.change_multi_assets_mode).
#[derive(Clone, Debug, TypedBuilder)]
pub struct ChangeMultiAssetsModeParams {
    /// "true": Multi-Assets Mode; "false": Single-Asset Mode
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub multi_assets_margin: String,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`change_position_mode`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`change_position_mode`](#method.change_position_mode).
#[derive(Clone, Debug, TypedBuilder)]
pub struct ChangePositionModeParams {
    /// "true": Hedge Mode; "false": One-way Mode
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub dual_side_position: String,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`current_all_open_orders`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`current_all_open_orders`](#method.current_all_open_orders).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct CurrentAllOpenOrdersParams {
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

/// Request parameters for the [`get_order_modify_history`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`get_order_modify_history`](#method.get_order_modify_history).
#[derive(Clone, Debug, TypedBuilder)]
pub struct GetOrderModifyHistoryParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    ///
    /// The `order_id` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub order_id: Option<i64>,
    ///
    /// The `orig_client_order_id` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub orig_client_order_id: Option<String>,
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

/// Request parameters for the [`get_position_margin_change_history`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`get_position_margin_change_history`](#method.get_position_margin_change_history).
#[derive(Clone, Debug, TypedBuilder)]
pub struct GetPositionMarginChangeHistoryParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// 1: Add position margin，2: Reduce position margin
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub r#type: Option<i64>,
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

/// Request parameters for the [`modify_isolated_position_margin`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`modify_isolated_position_margin`](#method.modify_isolated_position_margin).
#[derive(Clone, Debug, TypedBuilder)]
pub struct ModifyIsolatedPositionMarginParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    ///
    /// The `amount` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub amount: rust_decimal::Decimal,
    ///
    /// The `r#type` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub r#type: String,
    /// Default `BOTH` for One-way Mode ; `LONG` or `SHORT` for Hedge Mode. It must be sent with Hedge Mode.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub position_side: Option<ModifyIsolatedPositionMarginPositionSideEnum>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`modify_multiple_orders`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`modify_multiple_orders`](#method.modify_multiple_orders).
#[derive(Clone, Debug, TypedBuilder)]
pub struct ModifyMultipleOrdersParams {
    /// order list. Max 5 orders
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub batch_orders: Vec<models::ModifyMultipleOrdersBatchOrdersParameterInner>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`modify_order`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`modify_order`](#method.modify_order).
#[derive(Clone, Debug, TypedBuilder)]
pub struct ModifyOrderParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// `SELL`, `BUY`
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub side: ModifyOrderSideEnum,
    /// Order quantity, cannot be sent with `closePosition=true`
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub quantity: rust_decimal::Decimal,
    ///
    /// The `price` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub price: rust_decimal::Decimal,
    ///
    /// The `order_id` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub order_id: Option<i64>,
    ///
    /// The `orig_client_order_id` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub orig_client_order_id: Option<String>,
    /// only avaliable for `LIMIT`/`STOP`/`TAKE_PROFIT` order; can be set to `OPPONENT`/ `OPPONENT_5`/ `OPPONENT_10`/ `OPPONENT_20`: /`QUEUE`/ `QUEUE_5`/ `QUEUE_10`/ `QUEUE_20`; Can't be passed together with `price`
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub price_match: Option<ModifyOrderPriceMatchEnum>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`new_order`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`new_order`](#method.new_order).
#[derive(Clone, Debug, TypedBuilder)]
pub struct NewOrderParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// `SELL`, `BUY`
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub side: NewOrderSideEnum,
    ///
    /// The `r#type` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub r#type: String,
    /// Default `BOTH` for One-way Mode ; `LONG` or `SHORT` for Hedge Mode. It must be sent with Hedge Mode.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub position_side: Option<NewOrderPositionSideEnum>,
    ///
    /// The `time_in_force` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub time_in_force: Option<NewOrderTimeInForceEnum>,
    /// Cannot be sent with `closePosition`=`true`(Close-All)
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub quantity: Option<rust_decimal::Decimal>,
    /// "true" or "false". default "false". Cannot be sent in Hedge Mode; cannot be sent with `closePosition`=`true`
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub reduce_only: Option<String>,
    ///
    /// The `price` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub price: Option<rust_decimal::Decimal>,
    /// A unique id among open orders. Automatically generated if not sent. Can only be string following the rule: `^[\.A-Z\:/a-z0-9_-]{1,36}$`
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub new_client_order_id: Option<String>,
    /// Used with `STOP/STOP_MARKET` or `TAKE_PROFIT/TAKE_PROFIT_MARKET` orders.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub stop_price: Option<rust_decimal::Decimal>,
    /// `true`, `false`；Close-All，used with `STOP_MARKET` or `TAKE_PROFIT_MARKET`.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub close_position: Option<String>,
    /// Used with `TRAILING_STOP_MARKET` orders, default as the latest price(supporting different `workingType`)
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub activation_price: Option<rust_decimal::Decimal>,
    /// Used with `TRAILING_STOP_MARKET` orders, min 0.1, max 10 where 1 for 1%
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub callback_rate: Option<rust_decimal::Decimal>,
    /// stopPrice triggered by: "`MARK_PRICE`", "`CONTRACT_PRICE`". Default "`CONTRACT_PRICE`"
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub working_type: Option<NewOrderWorkingTypeEnum>,
    /// "TRUE" or "FALSE", default "FALSE". Used with `STOP/STOP_MARKET` or `TAKE_PROFIT/TAKE_PROFIT_MARKET` orders.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub price_protect: Option<String>,
    /// "ACK", "RESULT", default "ACK"
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub new_order_resp_type: Option<NewOrderNewOrderRespTypeEnum>,
    /// only avaliable for `LIMIT`/`STOP`/`TAKE_PROFIT` order; can be set to `OPPONENT`/ `OPPONENT_5`/ `OPPONENT_10`/ `OPPONENT_20`: /`QUEUE`/ `QUEUE_5`/ `QUEUE_10`/ `QUEUE_20`; Can't be passed together with `price`
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub price_match: Option<NewOrderPriceMatchEnum>,
    /// `NONE`:No STP / `EXPIRE_TAKER`:expire taker order when STP triggers/ `EXPIRE_MAKER`:expire taker order when STP triggers/ `EXPIRE_BOTH`:expire both orders when STP triggers; default `NONE`
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub self_trade_prevention_mode: Option<NewOrderSelfTradePreventionModeEnum>,
    /// order cancel time for timeInForce `GTD`, mandatory when `timeInforce` set to `GTD`; order the timestamp only retains second-level precision, ms part will be ignored; The goodTillDate timestamp must be greater than the current time plus 600 seconds and smaller than 253402300799000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub good_till_date: Option<i64>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`place_multiple_orders`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`place_multiple_orders`](#method.place_multiple_orders).
#[derive(Clone, Debug, TypedBuilder)]
pub struct PlaceMultipleOrdersParams {
    /// order list. Max 5 orders
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub batch_orders: Vec<models::PlaceMultipleOrdersBatchOrdersParameterInner>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`position_adl_quantile_estimation`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`position_adl_quantile_estimation`](#method.position_adl_quantile_estimation).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct PositionAdlQuantileEstimationParams {
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

/// Request parameters for the [`position_information_v2`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`position_information_v2`](#method.position_information_v2).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct PositionInformationV2Params {
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

/// Request parameters for the [`position_information_v3`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`position_information_v3`](#method.position_information_v3).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct PositionInformationV3Params {
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

/// Request parameters for the [`query_current_open_order`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`query_current_open_order`](#method.query_current_open_order).
#[derive(Clone, Debug, TypedBuilder)]
pub struct QueryCurrentOpenOrderParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    ///
    /// The `order_id` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub order_id: Option<i64>,
    ///
    /// The `orig_client_order_id` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub orig_client_order_id: Option<String>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`query_order`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`query_order`](#method.query_order).
#[derive(Clone, Debug, TypedBuilder)]
pub struct QueryOrderParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    ///
    /// The `order_id` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub order_id: Option<i64>,
    ///
    /// The `orig_client_order_id` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub orig_client_order_id: Option<String>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`test_order`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`test_order`](#method.test_order).
#[derive(Clone, Debug, TypedBuilder)]
pub struct TestOrderParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub symbol: String,
    /// `SELL`, `BUY`
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub side: TestOrderSideEnum,
    ///
    /// The `r#type` parameter.
    ///
    /// This field is **required.
    #[builder(setter(into))]
    pub r#type: String,
    /// Default `BOTH` for One-way Mode ; `LONG` or `SHORT` for Hedge Mode. It must be sent with Hedge Mode.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub position_side: Option<TestOrderPositionSideEnum>,
    ///
    /// The `time_in_force` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub time_in_force: Option<TestOrderTimeInForceEnum>,
    /// Cannot be sent with `closePosition`=`true`(Close-All)
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub quantity: Option<rust_decimal::Decimal>,
    /// "true" or "false". default "false". Cannot be sent in Hedge Mode; cannot be sent with `closePosition`=`true`
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub reduce_only: Option<String>,
    ///
    /// The `price` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub price: Option<rust_decimal::Decimal>,
    /// A unique id among open orders. Automatically generated if not sent. Can only be string following the rule: `^[\.A-Z\:/a-z0-9_-]{1,36}$`
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub new_client_order_id: Option<String>,
    /// Used with `STOP/STOP_MARKET` or `TAKE_PROFIT/TAKE_PROFIT_MARKET` orders.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub stop_price: Option<rust_decimal::Decimal>,
    /// `true`, `false`；Close-All，used with `STOP_MARKET` or `TAKE_PROFIT_MARKET`.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub close_position: Option<String>,
    /// Used with `TRAILING_STOP_MARKET` orders, default as the latest price(supporting different `workingType`)
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub activation_price: Option<rust_decimal::Decimal>,
    /// Used with `TRAILING_STOP_MARKET` orders, min 0.1, max 10 where 1 for 1%
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub callback_rate: Option<rust_decimal::Decimal>,
    /// stopPrice triggered by: "`MARK_PRICE`", "`CONTRACT_PRICE`". Default "`CONTRACT_PRICE`"
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub working_type: Option<TestOrderWorkingTypeEnum>,
    /// "TRUE" or "FALSE", default "FALSE". Used with `STOP/STOP_MARKET` or `TAKE_PROFIT/TAKE_PROFIT_MARKET` orders.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub price_protect: Option<String>,
    /// "ACK", "RESULT", default "ACK"
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub new_order_resp_type: Option<TestOrderNewOrderRespTypeEnum>,
    /// only avaliable for `LIMIT`/`STOP`/`TAKE_PROFIT` order; can be set to `OPPONENT`/ `OPPONENT_5`/ `OPPONENT_10`/ `OPPONENT_20`: /`QUEUE`/ `QUEUE_5`/ `QUEUE_10`/ `QUEUE_20`; Can't be passed together with `price`
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub price_match: Option<TestOrderPriceMatchEnum>,
    /// `NONE`:No STP / `EXPIRE_TAKER`:expire taker order when STP triggers/ `EXPIRE_MAKER`:expire taker order when STP triggers/ `EXPIRE_BOTH`:expire both orders when STP triggers; default `NONE`
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub self_trade_prevention_mode: Option<TestOrderSelfTradePreventionModeEnum>,
    /// order cancel time for timeInForce `GTD`, mandatory when `timeInforce` set to `GTD`; order the timestamp only retains second-level precision, ms part will be ignored; The goodTillDate timestamp must be greater than the current time plus 600 seconds and smaller than 253402300799000
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub good_till_date: Option<i64>,
    ///
    /// The `recv_window` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub recv_window: Option<i64>,
}

/// Request parameters for the [`users_force_orders`] operation.
///
/// This struct holds all of the inputs you can pass when calling
/// [`users_force_orders`](#method.users_force_orders).
#[derive(Clone, Debug, TypedBuilder, Default)]
pub struct UsersForceOrdersParams {
    ///
    /// The `symbol` parameter.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub symbol: Option<String>,
    /// "LIQUIDATION" for liquidation orders, "ADL" for ADL orders.
    ///
    /// This field is **optional.
    #[builder(setter(into), default)]
    pub auto_close_type: Option<UsersForceOrdersAutoCloseTypeEnum>,
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

#[async_trait]
impl TradeApi for TradeApiClient {
    async fn account_trade_list(
        &self,
        params: AccountTradeListParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::AccountTradeListResponseInner>>> {
        let AccountTradeListParams {
            symbol,
            order_id,
            start_time,
            end_time,
            from_id,
            limit,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = order_id {
            query_params.insert("orderId".to_string(), json!(rw));
        }

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        if let Some(rw) = from_id {
            query_params.insert("fromId".to_string(), json!(rw));
        }

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<Vec<models::AccountTradeListResponseInner>>(
            &self.configuration,
            "/fapi/v1/userTrades",
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

    async fn all_orders(
        &self,
        params: AllOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::AllOrdersResponseInner>>> {
        let AllOrdersParams {
            symbol,
            order_id,
            start_time,
            end_time,
            limit,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = order_id {
            query_params.insert("orderId".to_string(), json!(rw));
        }

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<Vec<models::AllOrdersResponseInner>>(
            &self.configuration,
            "/fapi/v1/allOrders",
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

    async fn auto_cancel_all_open_orders(
        &self,
        params: AutoCancelAllOpenOrdersParams,
    ) -> anyhow::Result<RestApiResponse<models::AutoCancelAllOpenOrdersResponse>> {
        let AutoCancelAllOpenOrdersParams {
            symbol,
            countdown_time,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("countdownTime".to_string(), json!(countdown_time));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::AutoCancelAllOpenOrdersResponse>(
            &self.configuration,
            "/fapi/v1/countdownCancelAll",
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

    async fn cancel_all_open_orders(
        &self,
        params: CancelAllOpenOrdersParams,
    ) -> anyhow::Result<RestApiResponse<models::CancelAllOpenOrdersResponse>> {
        let CancelAllOpenOrdersParams {
            symbol,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::CancelAllOpenOrdersResponse>(
            &self.configuration,
            "/fapi/v1/allOpenOrders",
            reqwest::Method::DELETE,
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

    async fn cancel_multiple_orders(
        &self,
        params: CancelMultipleOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::CancelMultipleOrdersResponseInner>>> {
        let CancelMultipleOrdersParams {
            symbol,
            order_id_list,
            orig_client_order_id_list,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = order_id_list {
            query_params.insert("orderIdList".to_string(), json!(rw));
        }

        if let Some(rw) = orig_client_order_id_list {
            query_params.insert("origClientOrderIdList".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<Vec<models::CancelMultipleOrdersResponseInner>>(
            &self.configuration,
            "/fapi/v1/batchOrders",
            reqwest::Method::DELETE,
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

    async fn cancel_order(
        &self,
        params: CancelOrderParams,
    ) -> anyhow::Result<RestApiResponse<models::CancelOrderResponse>> {
        let CancelOrderParams {
            symbol,
            order_id,
            orig_client_order_id,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = order_id {
            query_params.insert("orderId".to_string(), json!(rw));
        }

        if let Some(rw) = orig_client_order_id {
            query_params.insert("origClientOrderId".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::CancelOrderResponse>(
            &self.configuration,
            "/fapi/v1/order",
            reqwest::Method::DELETE,
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

    async fn change_initial_leverage(
        &self,
        params: ChangeInitialLeverageParams,
    ) -> anyhow::Result<RestApiResponse<models::ChangeInitialLeverageResponse>> {
        let ChangeInitialLeverageParams {
            symbol,
            leverage,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("leverage".to_string(), json!(leverage));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::ChangeInitialLeverageResponse>(
            &self.configuration,
            "/fapi/v1/leverage",
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

    async fn change_margin_type(
        &self,
        params: ChangeMarginTypeParams,
    ) -> anyhow::Result<RestApiResponse<models::ChangeMarginTypeResponse>> {
        let ChangeMarginTypeParams {
            symbol,
            margin_type,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("marginType".to_string(), json!(margin_type));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::ChangeMarginTypeResponse>(
            &self.configuration,
            "/fapi/v1/marginType",
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

    async fn change_multi_assets_mode(
        &self,
        params: ChangeMultiAssetsModeParams,
    ) -> anyhow::Result<RestApiResponse<models::ChangeMultiAssetsModeResponse>> {
        let ChangeMultiAssetsModeParams {
            multi_assets_margin,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("multiAssetsMargin".to_string(), json!(multi_assets_margin));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::ChangeMultiAssetsModeResponse>(
            &self.configuration,
            "/fapi/v1/multiAssetsMargin",
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

    async fn change_position_mode(
        &self,
        params: ChangePositionModeParams,
    ) -> anyhow::Result<RestApiResponse<models::ChangePositionModeResponse>> {
        let ChangePositionModeParams {
            dual_side_position,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("dualSidePosition".to_string(), json!(dual_side_position));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::ChangePositionModeResponse>(
            &self.configuration,
            "/fapi/v1/positionSide/dual",
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

    async fn current_all_open_orders(
        &self,
        params: CurrentAllOpenOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::AllOrdersResponseInner>>> {
        let CurrentAllOpenOrdersParams {
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

        send_request::<Vec<models::AllOrdersResponseInner>>(
            &self.configuration,
            "/fapi/v1/openOrders",
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

    async fn get_order_modify_history(
        &self,
        params: GetOrderModifyHistoryParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetOrderModifyHistoryResponseInner>>> {
        let GetOrderModifyHistoryParams {
            symbol,
            order_id,
            orig_client_order_id,
            start_time,
            end_time,
            limit,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = order_id {
            query_params.insert("orderId".to_string(), json!(rw));
        }

        if let Some(rw) = orig_client_order_id {
            query_params.insert("origClientOrderId".to_string(), json!(rw));
        }

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<Vec<models::GetOrderModifyHistoryResponseInner>>(
            &self.configuration,
            "/fapi/v1/orderAmendment",
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

    async fn get_position_margin_change_history(
        &self,
        params: GetPositionMarginChangeHistoryParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::GetPositionMarginChangeHistoryResponseInner>>> {
        let GetPositionMarginChangeHistoryParams {
            symbol,
            r#type,
            start_time,
            end_time,
            limit,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = r#type {
            query_params.insert("type".to_string(), json!(rw));
        }

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<Vec<models::GetPositionMarginChangeHistoryResponseInner>>(
            &self.configuration,
            "/fapi/v1/positionMargin/history",
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

    async fn modify_isolated_position_margin(
        &self,
        params: ModifyIsolatedPositionMarginParams,
    ) -> anyhow::Result<RestApiResponse<models::ModifyIsolatedPositionMarginResponse>> {
        let ModifyIsolatedPositionMarginParams {
            symbol,
            amount,
            r#type,
            position_side,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("amount".to_string(), json!(amount));

        query_params.insert("type".to_string(), json!(r#type));

        if let Some(rw) = position_side {
            query_params.insert("positionSide".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::ModifyIsolatedPositionMarginResponse>(
            &self.configuration,
            "/fapi/v1/positionMargin",
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

    async fn modify_multiple_orders(
        &self,
        params: ModifyMultipleOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::ModifyMultipleOrdersResponseInner>>> {
        let ModifyMultipleOrdersParams {
            batch_orders,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("batchOrders".to_string(), json!(batch_orders));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<Vec<models::ModifyMultipleOrdersResponseInner>>(
            &self.configuration,
            "/fapi/v1/batchOrders",
            reqwest::Method::PUT,
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

    async fn modify_order(
        &self,
        params: ModifyOrderParams,
    ) -> anyhow::Result<RestApiResponse<models::ModifyOrderResponse>> {
        let ModifyOrderParams {
            symbol,
            side,
            quantity,
            price,
            order_id,
            orig_client_order_id,
            price_match,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("side".to_string(), json!(side));

        query_params.insert("quantity".to_string(), json!(quantity));

        query_params.insert("price".to_string(), json!(price));

        if let Some(rw) = order_id {
            query_params.insert("orderId".to_string(), json!(rw));
        }

        if let Some(rw) = orig_client_order_id {
            query_params.insert("origClientOrderId".to_string(), json!(rw));
        }

        if let Some(rw) = price_match {
            query_params.insert("priceMatch".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::ModifyOrderResponse>(
            &self.configuration,
            "/fapi/v1/order",
            reqwest::Method::PUT,
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

    async fn new_order(&self, params: NewOrderParams) -> anyhow::Result<RestApiResponse<models::NewOrderResponse>> {
        let NewOrderParams {
            symbol,
            side,
            r#type,
            position_side,
            time_in_force,
            quantity,
            reduce_only,
            price,
            new_client_order_id,
            stop_price,
            close_position,
            activation_price,
            callback_rate,
            working_type,
            price_protect,
            new_order_resp_type,
            price_match,
            self_trade_prevention_mode,
            good_till_date,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("side".to_string(), json!(side));

        query_params.insert("type".to_string(), json!(r#type));

        if let Some(rw) = position_side {
            query_params.insert("positionSide".to_string(), json!(rw));
        }

        if let Some(rw) = time_in_force {
            query_params.insert("timeInForce".to_string(), json!(rw));
        }

        if let Some(rw) = quantity {
            query_params.insert("quantity".to_string(), json!(rw));
        }

        if let Some(rw) = reduce_only {
            query_params.insert("reduceOnly".to_string(), json!(rw));
        }

        if let Some(rw) = price {
            query_params.insert("price".to_string(), json!(rw));
        }

        if let Some(rw) = new_client_order_id {
            query_params.insert("newClientOrderId".to_string(), json!(rw));
        }

        if let Some(rw) = stop_price {
            query_params.insert("stopPrice".to_string(), json!(rw));
        }

        if let Some(rw) = close_position {
            query_params.insert("closePosition".to_string(), json!(rw));
        }

        if let Some(rw) = activation_price {
            query_params.insert("activationPrice".to_string(), json!(rw));
        }

        if let Some(rw) = callback_rate {
            query_params.insert("callbackRate".to_string(), json!(rw));
        }

        if let Some(rw) = working_type {
            query_params.insert("workingType".to_string(), json!(rw));
        }

        if let Some(rw) = price_protect {
            query_params.insert("priceProtect".to_string(), json!(rw));
        }

        if let Some(rw) = new_order_resp_type {
            query_params.insert("newOrderRespType".to_string(), json!(rw));
        }

        if let Some(rw) = price_match {
            query_params.insert("priceMatch".to_string(), json!(rw));
        }

        if let Some(rw) = self_trade_prevention_mode {
            query_params.insert("selfTradePreventionMode".to_string(), json!(rw));
        }

        if let Some(rw) = good_till_date {
            query_params.insert("goodTillDate".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::NewOrderResponse>(
            &self.configuration,
            "/fapi/v1/order",
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

    async fn place_multiple_orders(
        &self,
        params: PlaceMultipleOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::PlaceMultipleOrdersResponseInner>>> {
        let PlaceMultipleOrdersParams {
            batch_orders,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("batchOrders".to_string(), json!(batch_orders));

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<Vec<models::PlaceMultipleOrdersResponseInner>>(
            &self.configuration,
            "/fapi/v1/batchOrders",
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

    async fn position_adl_quantile_estimation(
        &self,
        params: PositionAdlQuantileEstimationParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::PositionAdlQuantileEstimationResponseInner>>> {
        let PositionAdlQuantileEstimationParams {
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

        send_request::<Vec<models::PositionAdlQuantileEstimationResponseInner>>(
            &self.configuration,
            "/fapi/v1/adlQuantile",
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

    async fn position_information_v2(
        &self,
        params: PositionInformationV2Params,
    ) -> anyhow::Result<RestApiResponse<Vec<models::PositionInformationV2ResponseInner>>> {
        let PositionInformationV2Params {
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

        send_request::<Vec<models::PositionInformationV2ResponseInner>>(
            &self.configuration,
            "/fapi/v2/positionRisk",
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

    async fn position_information_v3(
        &self,
        params: PositionInformationV3Params,
    ) -> anyhow::Result<RestApiResponse<Vec<models::PositionInformationV3ResponseInner>>> {
        let PositionInformationV3Params {
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

        send_request::<Vec<models::PositionInformationV3ResponseInner>>(
            &self.configuration,
            "/fapi/v3/positionRisk",
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

    async fn query_current_open_order(
        &self,
        params: QueryCurrentOpenOrderParams,
    ) -> anyhow::Result<RestApiResponse<models::QueryCurrentOpenOrderResponse>> {
        let QueryCurrentOpenOrderParams {
            symbol,
            order_id,
            orig_client_order_id,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = order_id {
            query_params.insert("orderId".to_string(), json!(rw));
        }

        if let Some(rw) = orig_client_order_id {
            query_params.insert("origClientOrderId".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::QueryCurrentOpenOrderResponse>(
            &self.configuration,
            "/fapi/v1/openOrder",
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

    async fn query_order(
        &self,
        params: QueryOrderParams,
    ) -> anyhow::Result<RestApiResponse<models::QueryOrderResponse>> {
        let QueryOrderParams {
            symbol,
            order_id,
            orig_client_order_id,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        if let Some(rw) = order_id {
            query_params.insert("orderId".to_string(), json!(rw));
        }

        if let Some(rw) = orig_client_order_id {
            query_params.insert("origClientOrderId".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::QueryOrderResponse>(
            &self.configuration,
            "/fapi/v1/order",
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

    async fn test_order(&self, params: TestOrderParams) -> anyhow::Result<RestApiResponse<models::TestOrderResponse>> {
        let TestOrderParams {
            symbol,
            side,
            r#type,
            position_side,
            time_in_force,
            quantity,
            reduce_only,
            price,
            new_client_order_id,
            stop_price,
            close_position,
            activation_price,
            callback_rate,
            working_type,
            price_protect,
            new_order_resp_type,
            price_match,
            self_trade_prevention_mode,
            good_till_date,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        query_params.insert("symbol".to_string(), json!(symbol));

        query_params.insert("side".to_string(), json!(side));

        query_params.insert("type".to_string(), json!(r#type));

        if let Some(rw) = position_side {
            query_params.insert("positionSide".to_string(), json!(rw));
        }

        if let Some(rw) = time_in_force {
            query_params.insert("timeInForce".to_string(), json!(rw));
        }

        if let Some(rw) = quantity {
            query_params.insert("quantity".to_string(), json!(rw));
        }

        if let Some(rw) = reduce_only {
            query_params.insert("reduceOnly".to_string(), json!(rw));
        }

        if let Some(rw) = price {
            query_params.insert("price".to_string(), json!(rw));
        }

        if let Some(rw) = new_client_order_id {
            query_params.insert("newClientOrderId".to_string(), json!(rw));
        }

        if let Some(rw) = stop_price {
            query_params.insert("stopPrice".to_string(), json!(rw));
        }

        if let Some(rw) = close_position {
            query_params.insert("closePosition".to_string(), json!(rw));
        }

        if let Some(rw) = activation_price {
            query_params.insert("activationPrice".to_string(), json!(rw));
        }

        if let Some(rw) = callback_rate {
            query_params.insert("callbackRate".to_string(), json!(rw));
        }

        if let Some(rw) = working_type {
            query_params.insert("workingType".to_string(), json!(rw));
        }

        if let Some(rw) = price_protect {
            query_params.insert("priceProtect".to_string(), json!(rw));
        }

        if let Some(rw) = new_order_resp_type {
            query_params.insert("newOrderRespType".to_string(), json!(rw));
        }

        if let Some(rw) = price_match {
            query_params.insert("priceMatch".to_string(), json!(rw));
        }

        if let Some(rw) = self_trade_prevention_mode {
            query_params.insert("selfTradePreventionMode".to_string(), json!(rw));
        }

        if let Some(rw) = good_till_date {
            query_params.insert("goodTillDate".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<models::TestOrderResponse>(
            &self.configuration,
            "/fapi/v1/order/test",
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

    async fn users_force_orders(
        &self,
        params: UsersForceOrdersParams,
    ) -> anyhow::Result<RestApiResponse<Vec<models::UsersForceOrdersResponseInner>>> {
        let UsersForceOrdersParams {
            symbol,
            auto_close_type,
            start_time,
            end_time,
            limit,
            recv_window,
        } = params;

        let mut query_params = BTreeMap::new();

        if let Some(rw) = symbol {
            query_params.insert("symbol".to_string(), json!(rw));
        }

        if let Some(rw) = auto_close_type {
            query_params.insert("autoCloseType".to_string(), json!(rw));
        }

        if let Some(rw) = start_time {
            query_params.insert("startTime".to_string(), json!(rw));
        }

        if let Some(rw) = end_time {
            query_params.insert("endTime".to_string(), json!(rw));
        }

        if let Some(rw) = limit {
            query_params.insert("limit".to_string(), json!(rw));
        }

        if let Some(rw) = recv_window {
            query_params.insert("recvWindow".to_string(), json!(rw));
        }

        send_request::<Vec<models::UsersForceOrdersResponseInner>>(
            &self.configuration,
            "/fapi/v1/forceOrders",
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

//     struct MockTradeApiClient {
//         force_error: bool,
//     }

//     #[async_trait]
//     impl TradeApi for MockTradeApiClient {
//         async fn account_trade_list(
//             &self,
//             _params: AccountTradeListParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::AccountTradeListResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"buyer":false,"commission":"-0.07819010","commissionAsset":"USDT","id":698759,"maker":false,"orderId":25851813,"price":"7819.01","qty":"0.002","quoteQty":"15.63802","realizedPnl":"-0.91539999","side":"SELL","positionSide":"SHORT","symbol":"BTCUSDT","time":1569514978020}]"#).unwrap();
//             let dummy_response: Vec<models::AccountTradeListResponseInner> = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into Vec<models::AccountTradeListResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn all_orders(
//             &self,
//             _params: AllOrdersParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::AllOrdersResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"avgPrice":"0.00000","clientOrderId":"abc","cumQuote":"0","executedQty":"0","orderId":1917641,"origQty":"0.40","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","time":1579276756075,"timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1579276756075,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0}]"#).unwrap();
//             let dummy_response: Vec<models::AllOrdersResponseInner> = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into Vec<models::AllOrdersResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn auto_cancel_all_open_orders(
//             &self,
//             _params: AutoCancelAllOpenOrdersParams,
//         ) -> anyhow::Result<RestApiResponse<models::AutoCancelAllOpenOrdersResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","countdownTime":"100000"}"#).unwrap();
//             let dummy_response: models::AutoCancelAllOpenOrdersResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::AutoCancelAllOpenOrdersResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn cancel_all_open_orders(
//             &self,
//             _params: CancelAllOpenOrdersParams,
//         ) -> anyhow::Result<RestApiResponse<models::CancelAllOpenOrdersResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"code":200,"msg":"The operation of cancel all open order is done."}"#)
//                     .unwrap();
//             let dummy_response: models::CancelAllOpenOrdersResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::CancelAllOpenOrdersResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn cancel_multiple_orders(
//             &self,
//             _params: CancelMultipleOrdersParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::CancelMultipleOrdersResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"clientOrderId":"myOrder1","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":283194212,"origQty":"11","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"CANCELED","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1571110484038,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000},{"code":-2011,"msg":"Unknown order sent."}]"#).unwrap();
//             let dummy_response: Vec<models::CancelMultipleOrdersResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::CancelMultipleOrdersResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn cancel_order(
//             &self,
//             _params: CancelOrderParams,
//         ) -> anyhow::Result<RestApiResponse<models::CancelOrderResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"clientOrderId":"myOrder1","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":283194212,"origQty":"11","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"CANCELED","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1571110484038,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000}"#).unwrap();
//             let dummy_response: models::CancelOrderResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::CancelOrderResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn change_initial_leverage(
//             &self,
//             _params: ChangeInitialLeverageParams,
//         ) -> anyhow::Result<RestApiResponse<models::ChangeInitialLeverageResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"leverage":21,"maxNotionalValue":"1000000","symbol":"BTCUSDT"}"#).unwrap();
//             let dummy_response: models::ChangeInitialLeverageResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::ChangeInitialLeverageResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn change_margin_type(
//             &self,
//             _params: ChangeMarginTypeParams,
//         ) -> anyhow::Result<RestApiResponse<models::ChangeMarginTypeResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"code":200,"msg":"success"}"#).unwrap();
//             let dummy_response: models::ChangeMarginTypeResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::ChangeMarginTypeResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn change_multi_assets_mode(
//             &self,
//             _params: ChangeMultiAssetsModeParams,
//         ) -> anyhow::Result<RestApiResponse<models::ChangeMultiAssetsModeResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"code":200,"msg":"success"}"#).unwrap();
//             let dummy_response: models::ChangeMultiAssetsModeResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::ChangeMultiAssetsModeResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn change_position_mode(
//             &self,
//             _params: ChangePositionModeParams,
//         ) -> anyhow::Result<RestApiResponse<models::ChangePositionModeResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"code":200,"msg":"success"}"#).unwrap();
//             let dummy_response: models::ChangePositionModeResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::ChangePositionModeResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn current_all_open_orders(
//             &self,
//             _params: CurrentAllOpenOrdersParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::AllOrdersResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"avgPrice":"0.00000","clientOrderId":"abc","cumQuote":"0","executedQty":"0","orderId":1917641,"origQty":"0.40","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","time":1579276756075,"timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1579276756075,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0}]"#).unwrap();
//             let dummy_response: Vec<models::AllOrdersResponseInner> = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into Vec<models::AllOrdersResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_order_modify_history(
//             &self,
//             _params: GetOrderModifyHistoryParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::GetOrderModifyHistoryResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"amendmentId":5363,"symbol":"BTCUSDT","pair":"BTCUSDT","orderId":20072994037,"clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","time":1629184560899,"amendment":{"price":{"before":"30004","after":"30003.2"},"origQty":{"before":"1","after":"1"},"count":3}},{"amendmentId":5361,"symbol":"BTCUSDT","pair":"BTCUSDT","orderId":20072994037,"clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","time":1629184533946,"amendment":{"price":{"before":"30005","after":"30004"},"origQty":{"before":"1","after":"1"},"count":2}},{"amendmentId":5325,"symbol":"BTCUSDT","pair":"BTCUSDT","orderId":20072994037,"clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","time":1629182711787,"amendment":{"price":{"before":"30002","after":"30005"},"origQty":{"before":"1","after":"1"},"count":1}}]"#).unwrap();
//             let dummy_response: Vec<models::GetOrderModifyHistoryResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::GetOrderModifyHistoryResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn get_position_margin_change_history(
//             &self,
//             _params: GetPositionMarginChangeHistoryParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::GetPositionMarginChangeHistoryResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","type":1,"deltaType":"USER_ADJUST","amount":"23.36332311","asset":"USDT","time":1578047897183,"positionSide":"BOTH"},{"symbol":"BTCUSDT","type":1,"deltaType":"USER_ADJUST","amount":"100","asset":"USDT","time":1578047900425,"positionSide":"LONG"}]"#).unwrap();
//             let dummy_response: Vec<models::GetPositionMarginChangeHistoryResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::GetPositionMarginChangeHistoryResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn modify_isolated_position_margin(
//             &self,
//             _params: ModifyIsolatedPositionMarginParams,
//         ) -> anyhow::Result<RestApiResponse<models::ModifyIsolatedPositionMarginResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"amount":100,"code":200,"msg":"Successfully modify position margin.","type":1}"#,
//             )
//             .unwrap();
//             let dummy_response: models::ModifyIsolatedPositionMarginResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::ModifyIsolatedPositionMarginResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn modify_multiple_orders(
//             &self,
//             _params: ModifyMultipleOrdersParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::ModifyMultipleOrdersResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"orderId":20072994037,"symbol":"BTCUSDT","pair":"BTCUSDT","status":"NEW","clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","price":"30005","avgPrice":"0.0","origQty":"1","executedQty":"0","cumQty":"0","cumBase":"0","timeInForce":"GTC","type":"LIMIT","reduceOnly":false,"closePosition":false,"side":"BUY","positionSide":"LONG","stopPrice":"0","workingType":"CONTRACT_PRICE","priceProtect":false,"origType":"LIMIT","priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0,"updateTime":1629182711600},{"code":-2022,"msg":"ReduceOnly Order is rejected."}]"#).unwrap();
//             let dummy_response: Vec<models::ModifyMultipleOrdersResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::ModifyMultipleOrdersResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn modify_order(
//             &self,
//             _params: ModifyOrderParams,
//         ) -> anyhow::Result<RestApiResponse<models::ModifyOrderResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"orderId":20072994037,"symbol":"BTCUSDT","pair":"BTCUSDT","status":"NEW","clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","price":"30005","avgPrice":"0.0","origQty":"1","executedQty":"0","cumQty":"0","cumBase":"0","timeInForce":"GTC","type":"LIMIT","reduceOnly":false,"closePosition":false,"side":"BUY","positionSide":"LONG","stopPrice":"0","workingType":"CONTRACT_PRICE","priceProtect":false,"origType":"LIMIT","priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0,"updateTime":1629182711600}"#).unwrap();
//             let dummy_response: models::ModifyOrderResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::ModifyOrderResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn new_order(
//             &self,
//             _params: NewOrderParams,
//         ) -> anyhow::Result<RestApiResponse<models::NewOrderResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"clientOrderId":"testOrder","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":22542179,"avgPrice":"0.00000","origQty":"10","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","timeInForce":"GTD","type":"TRAILING_STOP_MARKET","origType":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1566818724722,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000}"#).unwrap();
//             let dummy_response: models::NewOrderResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::NewOrderResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn place_multiple_orders(
//             &self,
//             _params: PlaceMultipleOrdersParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::PlaceMultipleOrdersResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"clientOrderId":"testOrder","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":22542179,"avgPrice":"0.00000","origQty":"10","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","symbol":"BTCUSDT","timeInForce":"GTC","type":"TRAILING_STOP_MARKET","origType":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1566818724722,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000},{"code":-2022,"msg":"ReduceOnly Order is rejected."}]"#).unwrap();
//             let dummy_response: Vec<models::PlaceMultipleOrdersResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::PlaceMultipleOrdersResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn position_adl_quantile_estimation(
//             &self,
//             _params: PositionAdlQuantileEstimationParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::PositionAdlQuantileEstimationResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"ETHUSDT","adlQuantile":{"LONG":3,"SHORT":3,"HEDGE":0}},{"symbol":"BTCUSDT","adlQuantile":{"LONG":1,"SHORT":2,"BOTH":0}}]"#).unwrap();
//             let dummy_response: Vec<models::PositionAdlQuantileEstimationResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::PositionAdlQuantileEstimationResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn position_information_v2(
//             &self,
//             _params: PositionInformationV2Params,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::PositionInformationV2ResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"entryPrice":"0.00000","breakEvenPrice":"0.0","marginType":"isolated","isAutoAddMargin":"false","isolatedMargin":"0.00000000","leverage":"10","liquidationPrice":"0","markPrice":"6679.50671178","maxNotionalValue":"20000000","positionAmt":"0.000","notional":"0","isolatedWallet":"0","symbol":"BTCUSDT","unRealizedProfit":"0.00000000","positionSide":"BOTH","updateTime":0},{"symbol":"BTCUSDT","positionAmt":"0.001","entryPrice":"22185.2","breakEvenPrice":"0.0","markPrice":"21123.05052574","unRealizedProfit":"-1.06214947","liquidationPrice":"19731.45529116","leverage":"4","maxNotionalValue":"100000000","marginType":"cross","isolatedMargin":"0.00000000","isAutoAddMargin":"false","positionSide":"LONG","notional":"21.12305052","isolatedWallet":"0","updateTime":1655217461579},{"symbol":"BTCUSDT","positionAmt":"0.000","entryPrice":"0.0","breakEvenPrice":"0.0","markPrice":"21123.05052574","unRealizedProfit":"0.00000000","liquidationPrice":"0","leverage":"4","maxNotionalValue":"100000000","marginType":"cross","isolatedMargin":"0.00000000","isAutoAddMargin":"false","positionSide":"SHORT","notional":"0","isolatedWallet":"0","updateTime":0}]"#).unwrap();
//             let dummy_response: Vec<models::PositionInformationV2ResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::PositionInformationV2ResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn position_information_v3(
//             &self,
//             _params: PositionInformationV3Params,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::PositionInformationV3ResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"ADAUSDT","positionSide":"BOTH","positionAmt":"30","entryPrice":"0.385","breakEvenPrice":"0.385077","markPrice":"0.41047590","unRealizedProfit":"0.76427700","liquidationPrice":"0","isolatedMargin":"0","notional":"12.31427700","marginAsset":"USDT","isolatedWallet":"0","initialMargin":"0.61571385","maintMargin":"0.08004280","positionInitialMargin":"0.61571385","openOrderInitialMargin":"0","adl":2,"bidNotional":"0","askNotional":"0","updateTime":1720736417660},{"symbol":"ADAUSDT","positionSide":"LONG","positionAmt":"30","entryPrice":"0.385","breakEvenPrice":"0.385077","markPrice":"0.41047590","unRealizedProfit":"0.76427700","liquidationPrice":"0","isolatedMargin":"0","notional":"12.31427700","marginAsset":"USDT","isolatedWallet":"0","initialMargin":"0.61571385","maintMargin":"0.08004280","positionInitialMargin":"0.61571385","openOrderInitialMargin":"0","adl":2,"bidNotional":"0","askNotional":"0","updateTime":1720736417660},{"symbol":"COMPUSDT","positionSide":"SHORT","positionAmt":"-1.000","entryPrice":"70.92841","breakEvenPrice":"70.900038636","markPrice":"49.72023376","unRealizedProfit":"21.20817624","liquidationPrice":"2260.56757210","isolatedMargin":"0","notional":"-49.72023376","marginAsset":"USDT","isolatedWallet":"0","initialMargin":"2.48601168","maintMargin":"0.49720233","positionInitialMargin":"2.48601168","openOrderInitialMargin":"0","adl":2,"bidNotional":"0","askNotional":"0","updateTime":1708943511656}]"#).unwrap();
//             let dummy_response: Vec<models::PositionInformationV3ResponseInner> =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into Vec<models::PositionInformationV3ResponseInner>");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn query_current_open_order(
//             &self,
//             _params: QueryCurrentOpenOrderParams,
//         ) -> anyhow::Result<RestApiResponse<models::QueryCurrentOpenOrderResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"avgPrice":"0.00000","clientOrderId":"abc","cumQuote":"0","executedQty":"0","orderId":1917641,"origQty":"0.40","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","time":1579276756075,"timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1579276756075,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0}"#).unwrap();
//             let dummy_response: models::QueryCurrentOpenOrderResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::QueryCurrentOpenOrderResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn query_order(
//             &self,
//             _params: QueryOrderParams,
//         ) -> anyhow::Result<RestApiResponse<models::QueryOrderResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"avgPrice":"0.00000","clientOrderId":"abc","cumQuote":"0","executedQty":"0","orderId":1917641,"origQty":"0.40","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","time":1579276756075,"timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1579276756075,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0}"#).unwrap();
//             let dummy_response: models::QueryOrderResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::QueryOrderResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn test_order(
//             &self,
//             _params: TestOrderParams,
//         ) -> anyhow::Result<RestApiResponse<models::TestOrderResponse>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"{"clientOrderId":"testOrder","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":22542179,"avgPrice":"0.00000","origQty":"10","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","timeInForce":"GTD","type":"TRAILING_STOP_MARKET","origType":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1566818724722,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000}"#).unwrap();
//             let dummy_response: models::TestOrderResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::TestOrderResponse");

//             let dummy = DummyRestApiResponse {
//                 inner: Box::new(move || Box::pin(async move { Ok(dummy_response) })),
//                 status: 200,
//                 headers: HashMap::new(),
//                 rate_limits: None,
//             };

//             Ok(dummy.into())
//         }

//         async fn users_force_orders(
//             &self,
//             _params: UsersForceOrdersParams,
//         ) -> anyhow::Result<RestApiResponse<Vec<models::UsersForceOrdersResponseInner>>> {
//             if self.force_error {
//                 return Err(ConnectorError::ConnectorClientError("ResponseError".to_string()).into());
//             }

//             let resp_json: Value = serde_json::from_str(r#"[{"orderId":6071832819,"symbol":"BTCUSDT","status":"FILLED","clientOrderId":"autoclose-1596107620040000020","price":"10871.09","avgPrice":"10913.21000","origQty":"0.001","executedQty":"0.001","cumQuote":"10.91321","timeInForce":"IOC","type":"LIMIT","reduceOnly":false,"closePosition":false,"side":"SELL","positionSide":"BOTH","stopPrice":"0","workingType":"CONTRACT_PRICE","origType":"LIMIT","time":1596107620044,"updateTime":1596107620087},{"orderId":6072734303,"symbol":"BTCUSDT","status":"FILLED","clientOrderId":"adl_autoclose","price":"11023.14","avgPrice":"10979.82000","origQty":"0.001","executedQty":"0.001","cumQuote":"10.97982","timeInForce":"GTC","type":"LIMIT","reduceOnly":false,"closePosition":false,"side":"BUY","positionSide":"SHORT","stopPrice":"0","workingType":"CONTRACT_PRICE","origType":"LIMIT","time":1596110725059,"updateTime":1596110725071}]"#).unwrap();
//             let dummy_response: Vec<models::UsersForceOrdersResponseInner> = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into Vec<models::UsersForceOrdersResponseInner>");

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
//     fn account_trade_list_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = AccountTradeListParams::builder("symbol_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"buyer":false,"commission":"-0.07819010","commissionAsset":"USDT","id":698759,"maker":false,"orderId":25851813,"price":"7819.01","qty":"0.002","quoteQty":"15.63802","realizedPnl":"-0.91539999","side":"SELL","positionSide":"SHORT","symbol":"BTCUSDT","time":1569514978020}]"#).unwrap();
//             let expected_response : Vec<models::AccountTradeListResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::AccountTradeListResponseInner>");

//             let resp = client.account_trade_list(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn account_trade_list_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = AccountTradeListParams::builder("symbol_example".to_string(),).order_id(1).start_time(1623319461670).end_time(1641782889000).from_id(1).limit(100).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"buyer":false,"commission":"-0.07819010","commissionAsset":"USDT","id":698759,"maker":false,"orderId":25851813,"price":"7819.01","qty":"0.002","quoteQty":"15.63802","realizedPnl":"-0.91539999","side":"SELL","positionSide":"SHORT","symbol":"BTCUSDT","time":1569514978020}]"#).unwrap();
//             let expected_response : Vec<models::AccountTradeListResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::AccountTradeListResponseInner>");

//             let resp = client.account_trade_list(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn account_trade_list_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = AccountTradeListParams::builder("symbol_example".to_string()).build().unwrap();

//             match client.account_trade_list(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn all_orders_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = AllOrdersParams::builder("symbol_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"avgPrice":"0.00000","clientOrderId":"abc","cumQuote":"0","executedQty":"0","orderId":1917641,"origQty":"0.40","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","time":1579276756075,"timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1579276756075,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0}]"#).unwrap();
//             let expected_response : Vec<models::AllOrdersResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::AllOrdersResponseInner>");

//             let resp = client.all_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn all_orders_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = AllOrdersParams::builder("symbol_example".to_string(),).order_id(1).start_time(1623319461670).end_time(1641782889000).limit(100).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"avgPrice":"0.00000","clientOrderId":"abc","cumQuote":"0","executedQty":"0","orderId":1917641,"origQty":"0.40","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","time":1579276756075,"timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1579276756075,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0}]"#).unwrap();
//             let expected_response : Vec<models::AllOrdersResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::AllOrdersResponseInner>");

//             let resp = client.all_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn all_orders_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = AllOrdersParams::builder("symbol_example".to_string()).build().unwrap();

//             match client.all_orders(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn auto_cancel_all_open_orders_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = AutoCancelAllOpenOrdersParams::builder("symbol_example".to_string(), 789)
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","countdownTime":"100000"}"#).unwrap();
//             let expected_response: models::AutoCancelAllOpenOrdersResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::AutoCancelAllOpenOrdersResponse");

//             let resp = client.auto_cancel_all_open_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn auto_cancel_all_open_orders_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = AutoCancelAllOpenOrdersParams::builder("symbol_example".to_string(), 789)
//                 .recv_window(5000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"symbol":"BTCUSDT","countdownTime":"100000"}"#).unwrap();
//             let expected_response: models::AutoCancelAllOpenOrdersResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::AutoCancelAllOpenOrdersResponse");

//             let resp = client.auto_cancel_all_open_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn auto_cancel_all_open_orders_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = AutoCancelAllOpenOrdersParams::builder("symbol_example".to_string(), 789)
//                 .build()
//                 .unwrap();

//             match client.auto_cancel_all_open_orders(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn cancel_all_open_orders_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = CancelAllOpenOrdersParams::builder("symbol_example".to_string())
//                 .build()
//                 .unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"code":200,"msg":"The operation of cancel all open order is done."}"#)
//                     .unwrap();
//             let expected_response: models::CancelAllOpenOrdersResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::CancelAllOpenOrdersResponse");

//             let resp = client.cancel_all_open_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn cancel_all_open_orders_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = CancelAllOpenOrdersParams::builder("symbol_example".to_string())
//                 .recv_window(5000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"code":200,"msg":"The operation of cancel all open order is done."}"#)
//                     .unwrap();
//             let expected_response: models::CancelAllOpenOrdersResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::CancelAllOpenOrdersResponse");

//             let resp = client.cancel_all_open_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn cancel_all_open_orders_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = CancelAllOpenOrdersParams::builder("symbol_example".to_string())
//                 .build()
//                 .unwrap();

//             match client.cancel_all_open_orders(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn cancel_multiple_orders_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = CancelMultipleOrdersParams::builder("symbol_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"clientOrderId":"myOrder1","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":283194212,"origQty":"11","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"CANCELED","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1571110484038,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000},{"code":-2011,"msg":"Unknown order sent."}]"#).unwrap();
//             let expected_response : Vec<models::CancelMultipleOrdersResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::CancelMultipleOrdersResponseInner>");

//             let resp = client.cancel_multiple_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn cancel_multiple_orders_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = CancelMultipleOrdersParams::builder("symbol_example".to_string(),).order_id_list([1234567,].to_vec()).orig_client_order_id_list(["my_id_1".to_string(),].to_vec()).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"clientOrderId":"myOrder1","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":283194212,"origQty":"11","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"CANCELED","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1571110484038,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000},{"code":-2011,"msg":"Unknown order sent."}]"#).unwrap();
//             let expected_response : Vec<models::CancelMultipleOrdersResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::CancelMultipleOrdersResponseInner>");

//             let resp = client.cancel_multiple_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn cancel_multiple_orders_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = CancelMultipleOrdersParams::builder("symbol_example".to_string())
//                 .build()
//                 .unwrap();

//             match client.cancel_multiple_orders(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn cancel_order_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = CancelOrderParams::builder("symbol_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"clientOrderId":"myOrder1","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":283194212,"origQty":"11","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"CANCELED","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1571110484038,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000}"#).unwrap();
//             let expected_response : models::CancelOrderResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::CancelOrderResponse");

//             let resp = client.cancel_order(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn cancel_order_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = CancelOrderParams::builder("symbol_example".to_string(),).order_id(1).orig_client_order_id("1".to_string()).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"clientOrderId":"myOrder1","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":283194212,"origQty":"11","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"CANCELED","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1571110484038,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000}"#).unwrap();
//             let expected_response : models::CancelOrderResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::CancelOrderResponse");

//             let resp = client.cancel_order(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn cancel_order_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = CancelOrderParams::builder("symbol_example".to_string()).build().unwrap();

//             match client.cancel_order(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn change_initial_leverage_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = ChangeInitialLeverageParams::builder("symbol_example".to_string(), 789)
//                 .build()
//                 .unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"leverage":21,"maxNotionalValue":"1000000","symbol":"BTCUSDT"}"#).unwrap();
//             let expected_response: models::ChangeInitialLeverageResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::ChangeInitialLeverageResponse");

//             let resp = client.change_initial_leverage(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn change_initial_leverage_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = ChangeInitialLeverageParams::builder("symbol_example".to_string(), 789)
//                 .recv_window(5000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value =
//                 serde_json::from_str(r#"{"leverage":21,"maxNotionalValue":"1000000","symbol":"BTCUSDT"}"#).unwrap();
//             let expected_response: models::ChangeInitialLeverageResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::ChangeInitialLeverageResponse");

//             let resp = client.change_initial_leverage(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn change_initial_leverage_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = ChangeInitialLeverageParams::builder("symbol_example".to_string(), 789)
//                 .build()
//                 .unwrap();

//             match client.change_initial_leverage(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn change_margin_type_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params =
//                 ChangeMarginTypeParams::builder("symbol_example".to_string(), ChangeMarginTypeMarginTypeEnum::Isolated)
//                     .build()
//                     .unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"code":200,"msg":"success"}"#).unwrap();
//             let expected_response: models::ChangeMarginTypeResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::ChangeMarginTypeResponse");

//             let resp = client.change_margin_type(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn change_margin_type_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params =
//                 ChangeMarginTypeParams::builder("symbol_example".to_string(), ChangeMarginTypeMarginTypeEnum::Isolated)
//                     .recv_window(5000)
//                     .build()
//                     .unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"code":200,"msg":"success"}"#).unwrap();
//             let expected_response: models::ChangeMarginTypeResponse =
//                 serde_json::from_value(resp_json.clone()).expect("should parse into models::ChangeMarginTypeResponse");

//             let resp = client.change_margin_type(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn change_margin_type_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params =
//                 ChangeMarginTypeParams::builder("symbol_example".to_string(), ChangeMarginTypeMarginTypeEnum::Isolated)
//                     .build()
//                     .unwrap();

//             match client.change_margin_type(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn change_multi_assets_mode_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = ChangeMultiAssetsModeParams::builder("multi_assets_margin_example".to_string())
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"code":200,"msg":"success"}"#).unwrap();
//             let expected_response: models::ChangeMultiAssetsModeResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::ChangeMultiAssetsModeResponse");

//             let resp = client.change_multi_assets_mode(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn change_multi_assets_mode_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = ChangeMultiAssetsModeParams::builder("multi_assets_margin_example".to_string())
//                 .recv_window(5000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"code":200,"msg":"success"}"#).unwrap();
//             let expected_response: models::ChangeMultiAssetsModeResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::ChangeMultiAssetsModeResponse");

//             let resp = client.change_multi_assets_mode(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn change_multi_assets_mode_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = ChangeMultiAssetsModeParams::builder("multi_assets_margin_example".to_string())
//                 .build()
//                 .unwrap();

//             match client.change_multi_assets_mode(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn change_position_mode_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = ChangePositionModeParams::builder("dual_side_position_example".to_string())
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"code":200,"msg":"success"}"#).unwrap();
//             let expected_response: models::ChangePositionModeResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::ChangePositionModeResponse");

//             let resp = client.change_position_mode(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn change_position_mode_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = ChangePositionModeParams::builder("dual_side_position_example".to_string())
//                 .recv_window(5000)
//                 .build()
//                 .unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"code":200,"msg":"success"}"#).unwrap();
//             let expected_response: models::ChangePositionModeResponse = serde_json::from_value(resp_json.clone())
//                 .expect("should parse into models::ChangePositionModeResponse");

//             let resp = client.change_position_mode(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn change_position_mode_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = ChangePositionModeParams::builder("dual_side_position_example".to_string())
//                 .build()
//                 .unwrap();

//             match client.change_position_mode(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn current_all_open_orders_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = CurrentAllOpenOrdersParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"avgPrice":"0.00000","clientOrderId":"abc","cumQuote":"0","executedQty":"0","orderId":1917641,"origQty":"0.40","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","time":1579276756075,"timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1579276756075,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0}]"#).unwrap();
//             let expected_response : Vec<models::AllOrdersResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::AllOrdersResponseInner>");

//             let resp = client.current_all_open_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn current_all_open_orders_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = CurrentAllOpenOrdersParams::builder().symbol("symbol_example".to_string()).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"avgPrice":"0.00000","clientOrderId":"abc","cumQuote":"0","executedQty":"0","orderId":1917641,"origQty":"0.40","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","time":1579276756075,"timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1579276756075,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0}]"#).unwrap();
//             let expected_response : Vec<models::AllOrdersResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::AllOrdersResponseInner>");

//             let resp = client.current_all_open_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn current_all_open_orders_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = CurrentAllOpenOrdersParams::builder().build().unwrap();

//             match client.current_all_open_orders(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_order_modify_history_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = GetOrderModifyHistoryParams::builder("symbol_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"amendmentId":5363,"symbol":"BTCUSDT","pair":"BTCUSDT","orderId":20072994037,"clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","time":1629184560899,"amendment":{"price":{"before":"30004","after":"30003.2"},"origQty":{"before":"1","after":"1"},"count":3}},{"amendmentId":5361,"symbol":"BTCUSDT","pair":"BTCUSDT","orderId":20072994037,"clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","time":1629184533946,"amendment":{"price":{"before":"30005","after":"30004"},"origQty":{"before":"1","after":"1"},"count":2}},{"amendmentId":5325,"symbol":"BTCUSDT","pair":"BTCUSDT","orderId":20072994037,"clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","time":1629182711787,"amendment":{"price":{"before":"30002","after":"30005"},"origQty":{"before":"1","after":"1"},"count":1}}]"#).unwrap();
//             let expected_response : Vec<models::GetOrderModifyHistoryResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::GetOrderModifyHistoryResponseInner>");

//             let resp = client.get_order_modify_history(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_order_modify_history_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = GetOrderModifyHistoryParams::builder("symbol_example".to_string(),).order_id(1).orig_client_order_id("1".to_string()).start_time(1623319461670).end_time(1641782889000).limit(100).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"amendmentId":5363,"symbol":"BTCUSDT","pair":"BTCUSDT","orderId":20072994037,"clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","time":1629184560899,"amendment":{"price":{"before":"30004","after":"30003.2"},"origQty":{"before":"1","after":"1"},"count":3}},{"amendmentId":5361,"symbol":"BTCUSDT","pair":"BTCUSDT","orderId":20072994037,"clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","time":1629184533946,"amendment":{"price":{"before":"30005","after":"30004"},"origQty":{"before":"1","after":"1"},"count":2}},{"amendmentId":5325,"symbol":"BTCUSDT","pair":"BTCUSDT","orderId":20072994037,"clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","time":1629182711787,"amendment":{"price":{"before":"30002","after":"30005"},"origQty":{"before":"1","after":"1"},"count":1}}]"#).unwrap();
//             let expected_response : Vec<models::GetOrderModifyHistoryResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::GetOrderModifyHistoryResponseInner>");

//             let resp = client.get_order_modify_history(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_order_modify_history_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = GetOrderModifyHistoryParams::builder("symbol_example".to_string())
//                 .build()
//                 .unwrap();

//             match client.get_order_modify_history(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn get_position_margin_change_history_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = GetPositionMarginChangeHistoryParams::builder("symbol_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","type":1,"deltaType":"USER_ADJUST","amount":"23.36332311","asset":"USDT","time":1578047897183,"positionSide":"BOTH"},{"symbol":"BTCUSDT","type":1,"deltaType":"USER_ADJUST","amount":"100","asset":"USDT","time":1578047900425,"positionSide":"LONG"}]"#).unwrap();
//             let expected_response : Vec<models::GetPositionMarginChangeHistoryResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::GetPositionMarginChangeHistoryResponseInner>");

//             let resp = client.get_position_margin_change_history(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_position_margin_change_history_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = GetPositionMarginChangeHistoryParams::builder("symbol_example".to_string(),).r#type(789).start_time(1623319461670).end_time(1641782889000).limit(100).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"BTCUSDT","type":1,"deltaType":"USER_ADJUST","amount":"23.36332311","asset":"USDT","time":1578047897183,"positionSide":"BOTH"},{"symbol":"BTCUSDT","type":1,"deltaType":"USER_ADJUST","amount":"100","asset":"USDT","time":1578047900425,"positionSide":"LONG"}]"#).unwrap();
//             let expected_response : Vec<models::GetPositionMarginChangeHistoryResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::GetPositionMarginChangeHistoryResponseInner>");

//             let resp = client.get_position_margin_change_history(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn get_position_margin_change_history_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = GetPositionMarginChangeHistoryParams::builder("symbol_example".to_string())
//                 .build()
//                 .unwrap();

//             match client.get_position_margin_change_history(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn modify_isolated_position_margin_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = ModifyIsolatedPositionMarginParams::builder(
//                 "symbol_example".to_string(),
//                 dec!(1.0),
//                 "r#type_example".to_string(),
//             )
//             .build()
//             .unwrap();

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"amount":100,"code":200,"msg":"Successfully modify position margin.","type":1}"#,
//             )
//             .unwrap();
//             let expected_response: models::ModifyIsolatedPositionMarginResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::ModifyIsolatedPositionMarginResponse");

//             let resp = client
//                 .modify_isolated_position_margin(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn modify_isolated_position_margin_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = ModifyIsolatedPositionMarginParams::builder(
//                 "symbol_example".to_string(),
//                 dec!(1.0),
//                 "r#type_example".to_string(),
//             )
//             .position_side(ModifyIsolatedPositionMarginPositionSideEnum::Both)
//             .recv_window(5000)
//             .build()
//             .unwrap();

//             let resp_json: Value = serde_json::from_str(
//                 r#"{"amount":100,"code":200,"msg":"Successfully modify position margin.","type":1}"#,
//             )
//             .unwrap();
//             let expected_response: models::ModifyIsolatedPositionMarginResponse =
//                 serde_json::from_value(resp_json.clone())
//                     .expect("should parse into models::ModifyIsolatedPositionMarginResponse");

//             let resp = client
//                 .modify_isolated_position_margin(params)
//                 .await
//                 .expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn modify_isolated_position_margin_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = ModifyIsolatedPositionMarginParams::builder(
//                 "symbol_example".to_string(),
//                 dec!(1.0),
//                 "r#type_example".to_string(),
//             )
//             .build()
//             .unwrap();

//             match client.modify_isolated_position_margin(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn modify_multiple_orders_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = ModifyMultipleOrdersParams::builder(vec![],).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"orderId":20072994037,"symbol":"BTCUSDT","pair":"BTCUSDT","status":"NEW","clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","price":"30005","avgPrice":"0.0","origQty":"1","executedQty":"0","cumQty":"0","cumBase":"0","timeInForce":"GTC","type":"LIMIT","reduceOnly":false,"closePosition":false,"side":"BUY","positionSide":"LONG","stopPrice":"0","workingType":"CONTRACT_PRICE","priceProtect":false,"origType":"LIMIT","priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0,"updateTime":1629182711600},{"code":-2022,"msg":"ReduceOnly Order is rejected."}]"#).unwrap();
//             let expected_response : Vec<models::ModifyMultipleOrdersResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::ModifyMultipleOrdersResponseInner>");

//             let resp = client.modify_multiple_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn modify_multiple_orders_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = ModifyMultipleOrdersParams::builder(vec![],).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"orderId":20072994037,"symbol":"BTCUSDT","pair":"BTCUSDT","status":"NEW","clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","price":"30005","avgPrice":"0.0","origQty":"1","executedQty":"0","cumQty":"0","cumBase":"0","timeInForce":"GTC","type":"LIMIT","reduceOnly":false,"closePosition":false,"side":"BUY","positionSide":"LONG","stopPrice":"0","workingType":"CONTRACT_PRICE","priceProtect":false,"origType":"LIMIT","priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0,"updateTime":1629182711600},{"code":-2022,"msg":"ReduceOnly Order is rejected."}]"#).unwrap();
//             let expected_response : Vec<models::ModifyMultipleOrdersResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::ModifyMultipleOrdersResponseInner>");

//             let resp = client.modify_multiple_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn modify_multiple_orders_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = ModifyMultipleOrdersParams::builder(vec![]).build().unwrap();

//             match client.modify_multiple_orders(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn modify_order_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = ModifyOrderParams::builder("symbol_example".to_string(),ModifyOrderSideEnum::Buy,dec!(1.0),dec!(1.0),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"orderId":20072994037,"symbol":"BTCUSDT","pair":"BTCUSDT","status":"NEW","clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","price":"30005","avgPrice":"0.0","origQty":"1","executedQty":"0","cumQty":"0","cumBase":"0","timeInForce":"GTC","type":"LIMIT","reduceOnly":false,"closePosition":false,"side":"BUY","positionSide":"LONG","stopPrice":"0","workingType":"CONTRACT_PRICE","priceProtect":false,"origType":"LIMIT","priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0,"updateTime":1629182711600}"#).unwrap();
//             let expected_response : models::ModifyOrderResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::ModifyOrderResponse");

//             let resp = client.modify_order(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn modify_order_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = ModifyOrderParams::builder("symbol_example".to_string(),ModifyOrderSideEnum::Buy,dec!(1.0),dec!(1.0),).order_id(1).orig_client_order_id("1".to_string()).price_match(ModifyOrderPriceMatchEnum::None).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"orderId":20072994037,"symbol":"BTCUSDT","pair":"BTCUSDT","status":"NEW","clientOrderId":"LJ9R4QZDihCaS8UAOOLpgW","price":"30005","avgPrice":"0.0","origQty":"1","executedQty":"0","cumQty":"0","cumBase":"0","timeInForce":"GTC","type":"LIMIT","reduceOnly":false,"closePosition":false,"side":"BUY","positionSide":"LONG","stopPrice":"0","workingType":"CONTRACT_PRICE","priceProtect":false,"origType":"LIMIT","priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0,"updateTime":1629182711600}"#).unwrap();
//             let expected_response : models::ModifyOrderResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::ModifyOrderResponse");

//             let resp = client.modify_order(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn modify_order_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = ModifyOrderParams::builder(
//                 "symbol_example".to_string(),
//                 ModifyOrderSideEnum::Buy,
//                 dec!(1.0),
//                 dec!(1.0),
//             )
//             .build()
//             .unwrap();

//             match client.modify_order(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn new_order_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = NewOrderParams::builder("symbol_example".to_string(),NewOrderSideEnum::Buy,"r#type_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"clientOrderId":"testOrder","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":22542179,"avgPrice":"0.00000","origQty":"10","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","timeInForce":"GTD","type":"TRAILING_STOP_MARKET","origType":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1566818724722,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000}"#).unwrap();
//             let expected_response : models::NewOrderResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::NewOrderResponse");

//             let resp = client.new_order(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn new_order_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = NewOrderParams::builder("symbol_example".to_string(),NewOrderSideEnum::Buy,"r#type_example".to_string(),).position_side(NewOrderPositionSideEnum::Both).time_in_force(NewOrderTimeInForceEnum::Gtc).quantity(dec!(1.0)).reduce_only("false".to_string()).price(dec!(1.0)).new_client_order_id("1".to_string()).stop_price(dec!(1.0)).close_position("close_position_example".to_string()).activation_price(dec!(1.0)).callback_rate(dec!(1.0)).working_type(NewOrderWorkingTypeEnum::MarkPrice).price_protect("false".to_string()).new_order_resp_type(NewOrderNewOrderRespTypeEnum::Ack).price_match(NewOrderPriceMatchEnum::None).self_trade_prevention_mode(NewOrderSelfTradePreventionModeEnum::ExpireTaker).good_till_date(789).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"clientOrderId":"testOrder","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":22542179,"avgPrice":"0.00000","origQty":"10","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","timeInForce":"GTD","type":"TRAILING_STOP_MARKET","origType":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1566818724722,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000}"#).unwrap();
//             let expected_response : models::NewOrderResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::NewOrderResponse");

//             let resp = client.new_order(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn new_order_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = NewOrderParams::builder(
//                 "symbol_example".to_string(),
//                 NewOrderSideEnum::Buy,
//                 "r#type_example".to_string(),
//             )
//             .build()
//             .unwrap();

//             match client.new_order(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn place_multiple_orders_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = PlaceMultipleOrdersParams::builder(vec![],).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"clientOrderId":"testOrder","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":22542179,"avgPrice":"0.00000","origQty":"10","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","symbol":"BTCUSDT","timeInForce":"GTC","type":"TRAILING_STOP_MARKET","origType":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1566818724722,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000},{"code":-2022,"msg":"ReduceOnly Order is rejected."}]"#).unwrap();
//             let expected_response : Vec<models::PlaceMultipleOrdersResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::PlaceMultipleOrdersResponseInner>");

//             let resp = client.place_multiple_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn place_multiple_orders_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = PlaceMultipleOrdersParams::builder(vec![],).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"clientOrderId":"testOrder","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":22542179,"avgPrice":"0.00000","origQty":"10","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","symbol":"BTCUSDT","timeInForce":"GTC","type":"TRAILING_STOP_MARKET","origType":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1566818724722,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000},{"code":-2022,"msg":"ReduceOnly Order is rejected."}]"#).unwrap();
//             let expected_response : Vec<models::PlaceMultipleOrdersResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::PlaceMultipleOrdersResponseInner>");

//             let resp = client.place_multiple_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn place_multiple_orders_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = PlaceMultipleOrdersParams::builder(vec![]).build().unwrap();

//             match client.place_multiple_orders(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn position_adl_quantile_estimation_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = PositionAdlQuantileEstimationParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"ETHUSDT","adlQuantile":{"LONG":3,"SHORT":3,"HEDGE":0}},{"symbol":"BTCUSDT","adlQuantile":{"LONG":1,"SHORT":2,"BOTH":0}}]"#).unwrap();
//             let expected_response : Vec<models::PositionAdlQuantileEstimationResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::PositionAdlQuantileEstimationResponseInner>");

//             let resp = client.position_adl_quantile_estimation(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn position_adl_quantile_estimation_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = PositionAdlQuantileEstimationParams::builder().symbol("symbol_example".to_string()).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"ETHUSDT","adlQuantile":{"LONG":3,"SHORT":3,"HEDGE":0}},{"symbol":"BTCUSDT","adlQuantile":{"LONG":1,"SHORT":2,"BOTH":0}}]"#).unwrap();
//             let expected_response : Vec<models::PositionAdlQuantileEstimationResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::PositionAdlQuantileEstimationResponseInner>");

//             let resp = client.position_adl_quantile_estimation(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn position_adl_quantile_estimation_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = PositionAdlQuantileEstimationParams::builder().build().unwrap();

//             match client.position_adl_quantile_estimation(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn position_information_v2_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = PositionInformationV2Params::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"entryPrice":"0.00000","breakEvenPrice":"0.0","marginType":"isolated","isAutoAddMargin":"false","isolatedMargin":"0.00000000","leverage":"10","liquidationPrice":"0","markPrice":"6679.50671178","maxNotionalValue":"20000000","positionAmt":"0.000","notional":"0","isolatedWallet":"0","symbol":"BTCUSDT","unRealizedProfit":"0.00000000","positionSide":"BOTH","updateTime":0},{"symbol":"BTCUSDT","positionAmt":"0.001","entryPrice":"22185.2","breakEvenPrice":"0.0","markPrice":"21123.05052574","unRealizedProfit":"-1.06214947","liquidationPrice":"19731.45529116","leverage":"4","maxNotionalValue":"100000000","marginType":"cross","isolatedMargin":"0.00000000","isAutoAddMargin":"false","positionSide":"LONG","notional":"21.12305052","isolatedWallet":"0","updateTime":1655217461579},{"symbol":"BTCUSDT","positionAmt":"0.000","entryPrice":"0.0","breakEvenPrice":"0.0","markPrice":"21123.05052574","unRealizedProfit":"0.00000000","liquidationPrice":"0","leverage":"4","maxNotionalValue":"100000000","marginType":"cross","isolatedMargin":"0.00000000","isAutoAddMargin":"false","positionSide":"SHORT","notional":"0","isolatedWallet":"0","updateTime":0}]"#).unwrap();
//             let expected_response : Vec<models::PositionInformationV2ResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::PositionInformationV2ResponseInner>");

//             let resp = client.position_information_v2(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn position_information_v2_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = PositionInformationV2Params::builder().symbol("symbol_example".to_string()).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"entryPrice":"0.00000","breakEvenPrice":"0.0","marginType":"isolated","isAutoAddMargin":"false","isolatedMargin":"0.00000000","leverage":"10","liquidationPrice":"0","markPrice":"6679.50671178","maxNotionalValue":"20000000","positionAmt":"0.000","notional":"0","isolatedWallet":"0","symbol":"BTCUSDT","unRealizedProfit":"0.00000000","positionSide":"BOTH","updateTime":0},{"symbol":"BTCUSDT","positionAmt":"0.001","entryPrice":"22185.2","breakEvenPrice":"0.0","markPrice":"21123.05052574","unRealizedProfit":"-1.06214947","liquidationPrice":"19731.45529116","leverage":"4","maxNotionalValue":"100000000","marginType":"cross","isolatedMargin":"0.00000000","isAutoAddMargin":"false","positionSide":"LONG","notional":"21.12305052","isolatedWallet":"0","updateTime":1655217461579},{"symbol":"BTCUSDT","positionAmt":"0.000","entryPrice":"0.0","breakEvenPrice":"0.0","markPrice":"21123.05052574","unRealizedProfit":"0.00000000","liquidationPrice":"0","leverage":"4","maxNotionalValue":"100000000","marginType":"cross","isolatedMargin":"0.00000000","isAutoAddMargin":"false","positionSide":"SHORT","notional":"0","isolatedWallet":"0","updateTime":0}]"#).unwrap();
//             let expected_response : Vec<models::PositionInformationV2ResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::PositionInformationV2ResponseInner>");

//             let resp = client.position_information_v2(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn position_information_v2_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = PositionInformationV2Params::builder().build().unwrap();

//             match client.position_information_v2(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn position_information_v3_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = PositionInformationV3Params::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"ADAUSDT","positionSide":"BOTH","positionAmt":"30","entryPrice":"0.385","breakEvenPrice":"0.385077","markPrice":"0.41047590","unRealizedProfit":"0.76427700","liquidationPrice":"0","isolatedMargin":"0","notional":"12.31427700","marginAsset":"USDT","isolatedWallet":"0","initialMargin":"0.61571385","maintMargin":"0.08004280","positionInitialMargin":"0.61571385","openOrderInitialMargin":"0","adl":2,"bidNotional":"0","askNotional":"0","updateTime":1720736417660},{"symbol":"ADAUSDT","positionSide":"LONG","positionAmt":"30","entryPrice":"0.385","breakEvenPrice":"0.385077","markPrice":"0.41047590","unRealizedProfit":"0.76427700","liquidationPrice":"0","isolatedMargin":"0","notional":"12.31427700","marginAsset":"USDT","isolatedWallet":"0","initialMargin":"0.61571385","maintMargin":"0.08004280","positionInitialMargin":"0.61571385","openOrderInitialMargin":"0","adl":2,"bidNotional":"0","askNotional":"0","updateTime":1720736417660},{"symbol":"COMPUSDT","positionSide":"SHORT","positionAmt":"-1.000","entryPrice":"70.92841","breakEvenPrice":"70.900038636","markPrice":"49.72023376","unRealizedProfit":"21.20817624","liquidationPrice":"2260.56757210","isolatedMargin":"0","notional":"-49.72023376","marginAsset":"USDT","isolatedWallet":"0","initialMargin":"2.48601168","maintMargin":"0.49720233","positionInitialMargin":"2.48601168","openOrderInitialMargin":"0","adl":2,"bidNotional":"0","askNotional":"0","updateTime":1708943511656}]"#).unwrap();
//             let expected_response : Vec<models::PositionInformationV3ResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::PositionInformationV3ResponseInner>");

//             let resp = client.position_information_v3(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn position_information_v3_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = PositionInformationV3Params::builder().symbol("symbol_example".to_string()).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"symbol":"ADAUSDT","positionSide":"BOTH","positionAmt":"30","entryPrice":"0.385","breakEvenPrice":"0.385077","markPrice":"0.41047590","unRealizedProfit":"0.76427700","liquidationPrice":"0","isolatedMargin":"0","notional":"12.31427700","marginAsset":"USDT","isolatedWallet":"0","initialMargin":"0.61571385","maintMargin":"0.08004280","positionInitialMargin":"0.61571385","openOrderInitialMargin":"0","adl":2,"bidNotional":"0","askNotional":"0","updateTime":1720736417660},{"symbol":"ADAUSDT","positionSide":"LONG","positionAmt":"30","entryPrice":"0.385","breakEvenPrice":"0.385077","markPrice":"0.41047590","unRealizedProfit":"0.76427700","liquidationPrice":"0","isolatedMargin":"0","notional":"12.31427700","marginAsset":"USDT","isolatedWallet":"0","initialMargin":"0.61571385","maintMargin":"0.08004280","positionInitialMargin":"0.61571385","openOrderInitialMargin":"0","adl":2,"bidNotional":"0","askNotional":"0","updateTime":1720736417660},{"symbol":"COMPUSDT","positionSide":"SHORT","positionAmt":"-1.000","entryPrice":"70.92841","breakEvenPrice":"70.900038636","markPrice":"49.72023376","unRealizedProfit":"21.20817624","liquidationPrice":"2260.56757210","isolatedMargin":"0","notional":"-49.72023376","marginAsset":"USDT","isolatedWallet":"0","initialMargin":"2.48601168","maintMargin":"0.49720233","positionInitialMargin":"2.48601168","openOrderInitialMargin":"0","adl":2,"bidNotional":"0","askNotional":"0","updateTime":1708943511656}]"#).unwrap();
//             let expected_response : Vec<models::PositionInformationV3ResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::PositionInformationV3ResponseInner>");

//             let resp = client.position_information_v3(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn position_information_v3_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = PositionInformationV3Params::builder().build().unwrap();

//             match client.position_information_v3(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn query_current_open_order_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = QueryCurrentOpenOrderParams::builder("symbol_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"avgPrice":"0.00000","clientOrderId":"abc","cumQuote":"0","executedQty":"0","orderId":1917641,"origQty":"0.40","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","time":1579276756075,"timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1579276756075,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0}"#).unwrap();
//             let expected_response : models::QueryCurrentOpenOrderResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::QueryCurrentOpenOrderResponse");

//             let resp = client.query_current_open_order(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn query_current_open_order_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = QueryCurrentOpenOrderParams::builder("symbol_example".to_string(),).order_id(1).orig_client_order_id("1".to_string()).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"avgPrice":"0.00000","clientOrderId":"abc","cumQuote":"0","executedQty":"0","orderId":1917641,"origQty":"0.40","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","time":1579276756075,"timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1579276756075,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0}"#).unwrap();
//             let expected_response : models::QueryCurrentOpenOrderResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::QueryCurrentOpenOrderResponse");

//             let resp = client.query_current_open_order(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn query_current_open_order_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = QueryCurrentOpenOrderParams::builder("symbol_example".to_string())
//                 .build()
//                 .unwrap();

//             match client.query_current_open_order(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn query_order_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = QueryOrderParams::builder("symbol_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"avgPrice":"0.00000","clientOrderId":"abc","cumQuote":"0","executedQty":"0","orderId":1917641,"origQty":"0.40","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","time":1579276756075,"timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1579276756075,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0}"#).unwrap();
//             let expected_response : models::QueryOrderResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::QueryOrderResponse");

//             let resp = client.query_order(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn query_order_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = QueryOrderParams::builder("symbol_example".to_string(),).order_id(1).orig_client_order_id("1".to_string()).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"avgPrice":"0.00000","clientOrderId":"abc","cumQuote":"0","executedQty":"0","orderId":1917641,"origQty":"0.40","origType":"TRAILING_STOP_MARKET","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","time":1579276756075,"timeInForce":"GTC","type":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1579276756075,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":0}"#).unwrap();
//             let expected_response : models::QueryOrderResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::QueryOrderResponse");

//             let resp = client.query_order(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn query_order_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = QueryOrderParams::builder("symbol_example".to_string()).build().unwrap();

//             match client.query_order(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn test_order_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = TestOrderParams::builder("symbol_example".to_string(),TestOrderSideEnum::Buy,"r#type_example".to_string(),).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"clientOrderId":"testOrder","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":22542179,"avgPrice":"0.00000","origQty":"10","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","timeInForce":"GTD","type":"TRAILING_STOP_MARKET","origType":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1566818724722,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000}"#).unwrap();
//             let expected_response : models::TestOrderResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::TestOrderResponse");

//             let resp = client.test_order(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn test_order_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = TestOrderParams::builder("symbol_example".to_string(),TestOrderSideEnum::Buy,"r#type_example".to_string(),).position_side(TestOrderPositionSideEnum::Both).time_in_force(TestOrderTimeInForceEnum::Gtc).quantity(dec!(1.0)).reduce_only("false".to_string()).price(dec!(1.0)).new_client_order_id("1".to_string()).stop_price(dec!(1.0)).close_position("close_position_example".to_string()).activation_price(dec!(1.0)).callback_rate(dec!(1.0)).working_type(TestOrderWorkingTypeEnum::MarkPrice).price_protect("false".to_string()).new_order_resp_type(TestOrderNewOrderRespTypeEnum::Ack).price_match(TestOrderPriceMatchEnum::None).self_trade_prevention_mode(TestOrderSelfTradePreventionModeEnum::ExpireTaker).good_till_date(789).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"{"clientOrderId":"testOrder","cumQty":"0","cumQuote":"0","executedQty":"0","orderId":22542179,"avgPrice":"0.00000","origQty":"10","price":"0","reduceOnly":false,"side":"BUY","positionSide":"SHORT","status":"NEW","stopPrice":"9300","closePosition":false,"symbol":"BTCUSDT","timeInForce":"GTD","type":"TRAILING_STOP_MARKET","origType":"TRAILING_STOP_MARKET","activatePrice":"9020","priceRate":"0.3","updateTime":1566818724722,"workingType":"CONTRACT_PRICE","priceProtect":false,"priceMatch":"NONE","selfTradePreventionMode":"NONE","goodTillDate":1693207680000}"#).unwrap();
//             let expected_response : models::TestOrderResponse = serde_json::from_value(resp_json.clone()).expect("should parse into models::TestOrderResponse");

//             let resp = client.test_order(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn test_order_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = TestOrderParams::builder(
//                 "symbol_example".to_string(),
//                 TestOrderSideEnum::Buy,
//                 "r#type_example".to_string(),
//             )
//             .build()
//             .unwrap();

//             match client.test_order(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }

//     #[test]
//     fn users_force_orders_required_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = UsersForceOrdersParams::builder().build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"orderId":6071832819,"symbol":"BTCUSDT","status":"FILLED","clientOrderId":"autoclose-1596107620040000020","price":"10871.09","avgPrice":"10913.21000","origQty":"0.001","executedQty":"0.001","cumQuote":"10.91321","timeInForce":"IOC","type":"LIMIT","reduceOnly":false,"closePosition":false,"side":"SELL","positionSide":"BOTH","stopPrice":"0","workingType":"CONTRACT_PRICE","origType":"LIMIT","time":1596107620044,"updateTime":1596107620087},{"orderId":6072734303,"symbol":"BTCUSDT","status":"FILLED","clientOrderId":"adl_autoclose","price":"11023.14","avgPrice":"10979.82000","origQty":"0.001","executedQty":"0.001","cumQuote":"10.97982","timeInForce":"GTC","type":"LIMIT","reduceOnly":false,"closePosition":false,"side":"BUY","positionSide":"SHORT","stopPrice":"0","workingType":"CONTRACT_PRICE","origType":"LIMIT","time":1596110725059,"updateTime":1596110725071}]"#).unwrap();
//             let expected_response : Vec<models::UsersForceOrdersResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::UsersForceOrdersResponseInner>");

//             let resp = client.users_force_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn users_force_orders_optional_params_success() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: false };

//             let params = UsersForceOrdersParams::builder().symbol("symbol_example".to_string()).auto_close_type(UsersForceOrdersAutoCloseTypeEnum::Liquidation).start_time(1623319461670).end_time(1641782889000).limit(100).recv_window(5000).build().unwrap();

//             let resp_json: Value = serde_json::from_str(r#"[{"orderId":6071832819,"symbol":"BTCUSDT","status":"FILLED","clientOrderId":"autoclose-1596107620040000020","price":"10871.09","avgPrice":"10913.21000","origQty":"0.001","executedQty":"0.001","cumQuote":"10.91321","timeInForce":"IOC","type":"LIMIT","reduceOnly":false,"closePosition":false,"side":"SELL","positionSide":"BOTH","stopPrice":"0","workingType":"CONTRACT_PRICE","origType":"LIMIT","time":1596107620044,"updateTime":1596107620087},{"orderId":6072734303,"symbol":"BTCUSDT","status":"FILLED","clientOrderId":"adl_autoclose","price":"11023.14","avgPrice":"10979.82000","origQty":"0.001","executedQty":"0.001","cumQuote":"10.97982","timeInForce":"GTC","type":"LIMIT","reduceOnly":false,"closePosition":false,"side":"BUY","positionSide":"SHORT","stopPrice":"0","workingType":"CONTRACT_PRICE","origType":"LIMIT","time":1596110725059,"updateTime":1596110725071}]"#).unwrap();
//             let expected_response : Vec<models::UsersForceOrdersResponseInner> = serde_json::from_value(resp_json.clone()).expect("should parse into Vec<models::UsersForceOrdersResponseInner>");

//             let resp = client.users_force_orders(params).await.expect("Expected a response");
//             let data_future = resp.data();
//             let actual_response = data_future.await.unwrap();
//             assert_eq!(actual_response, expected_response);
//         });
//     }

//     #[test]
//     fn users_force_orders_response_error() {
//         TOKIO_SHARED_RT.block_on(async {
//             let client = MockTradeApiClient { force_error: true };

//             let params = UsersForceOrdersParams::builder().build().unwrap();

//             match client.users_force_orders(params).await {
//                 Ok(_) => panic!("Expected an error"),
//                 Err(err) => {
//                     assert_eq!(err.to_string(), "Connector client error: ResponseError");
//                 }
//             }
//         });
//     }
// }
