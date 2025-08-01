#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaceMultipleOrdersBatchOrdersParameterInner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "side", skip_serializing_if = "Option::is_none")]
    pub side: Option<SideEnum>,
    #[serde(rename = "positionSide", skip_serializing_if = "Option::is_none")]
    pub position_side: Option<PositionSideEnum>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "timeInForce", skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForceEnum>,
    #[serde(rename = "quantity", skip_serializing_if = "Option::is_none")]
    pub quantity: Option<rust_decimal::Decimal>,
    #[serde(rename = "reduceOnly", skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<String>,
    #[serde(rename = "price", skip_serializing_if = "Option::is_none")]
    pub price: Option<rust_decimal::Decimal>,
    #[serde(rename = "newClientOrderId", skip_serializing_if = "Option::is_none")]
    pub new_client_order_id: Option<String>,
    #[serde(rename = "stopPrice", skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<rust_decimal::Decimal>,
    #[serde(rename = "activationPrice", skip_serializing_if = "Option::is_none")]
    pub activation_price: Option<rust_decimal::Decimal>,
    #[serde(rename = "callbackRate", skip_serializing_if = "Option::is_none")]
    pub callback_rate: Option<rust_decimal::Decimal>,
    #[serde(rename = "workingType", skip_serializing_if = "Option::is_none")]
    pub working_type: Option<WorkingTypeEnum>,
    #[serde(rename = "priceProtect", skip_serializing_if = "Option::is_none")]
    pub price_protect: Option<String>,
    #[serde(rename = "newOrderRespType", skip_serializing_if = "Option::is_none")]
    pub new_order_resp_type: Option<NewOrderRespTypeEnum>,
    #[serde(rename = "priceMatch", skip_serializing_if = "Option::is_none")]
    pub price_match: Option<PriceMatchEnum>,
    #[serde(
        rename = "selfTradePreventionMode",
        skip_serializing_if = "Option::is_none"
    )]
    pub self_trade_prevention_mode: Option<SelfTradePreventionModeEnum>,
    #[serde(rename = "goodTillDate", skip_serializing_if = "Option::is_none")]
    pub good_till_date: Option<i64>,
}

impl PlaceMultipleOrdersBatchOrdersParameterInner {
    #[must_use]
    pub fn new() -> PlaceMultipleOrdersBatchOrdersParameterInner {
        PlaceMultipleOrdersBatchOrdersParameterInner {
            symbol: None,
            side: None,
            position_side: None,
            r#type: None,
            time_in_force: None,
            quantity: None,
            reduce_only: None,
            price: None,
            new_client_order_id: None,
            stop_price: None,
            activation_price: None,
            callback_rate: None,
            working_type: None,
            price_protect: None,
            new_order_resp_type: None,
            price_match: None,
            self_trade_prevention_mode: None,
            good_till_date: None,
        }
    }
}
///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum SideEnum {
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
}

impl Default for SideEnum {
    fn default() -> SideEnum {
        Self::Buy
    }
}
///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum PositionSideEnum {
    #[serde(rename = "BOTH")]
    Both,
    #[serde(rename = "LONG")]
    Long,
    #[serde(rename = "SHORT")]
    Short,
}

impl Default for PositionSideEnum {
    fn default() -> PositionSideEnum {
        Self::Both
    }
}
///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum TimeInForceEnum {
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

impl Default for TimeInForceEnum {
    fn default() -> TimeInForceEnum {
        Self::Gtc
    }
}
///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum WorkingTypeEnum {
    #[serde(rename = "MARK_PRICE")]
    MarkPrice,
    #[serde(rename = "CONTRACT_PRICE")]
    ContractPrice,
}

impl Default for WorkingTypeEnum {
    fn default() -> WorkingTypeEnum {
        Self::MarkPrice
    }
}
///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum NewOrderRespTypeEnum {
    #[serde(rename = "ACK")]
    Ack,
    #[serde(rename = "RESULT")]
    Result,
}

impl Default for NewOrderRespTypeEnum {
    fn default() -> NewOrderRespTypeEnum {
        Self::Ack
    }
}
///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum PriceMatchEnum {
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

impl Default for PriceMatchEnum {
    fn default() -> PriceMatchEnum {
        Self::None
    }
}
///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum SelfTradePreventionModeEnum {
    #[serde(rename = "EXPIRE_TAKER")]
    ExpireTaker,
    #[serde(rename = "EXPIRE_BOTH")]
    ExpireBoth,
    #[serde(rename = "EXPIRE_MAKER")]
    ExpireMaker,
}

impl Default for SelfTradePreventionModeEnum {
    fn default() -> SelfTradePreventionModeEnum {
        Self::ExpireTaker
    }
}
