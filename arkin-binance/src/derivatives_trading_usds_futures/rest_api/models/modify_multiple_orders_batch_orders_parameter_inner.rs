#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModifyMultipleOrdersBatchOrdersParameterInner {
    #[serde(rename = "orderId", skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    #[serde(rename = "origClientOrderId", skip_serializing_if = "Option::is_none")]
    pub orig_client_order_id: Option<String>,
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "side", skip_serializing_if = "Option::is_none")]
    pub side: Option<SideEnum>,
    #[serde(rename = "quantity", skip_serializing_if = "Option::is_none")]
    pub quantity: Option<rust_decimal::Decimal>,
    #[serde(rename = "price", skip_serializing_if = "Option::is_none")]
    pub price: Option<rust_decimal::Decimal>,
    #[serde(rename = "priceMatch", skip_serializing_if = "Option::is_none")]
    pub price_match: Option<PriceMatchEnum>,
    #[serde(rename = "recvWindow", skip_serializing_if = "Option::is_none")]
    pub recv_window: Option<i64>,
}

impl ModifyMultipleOrdersBatchOrdersParameterInner {
    #[must_use]
    pub fn new() -> ModifyMultipleOrdersBatchOrdersParameterInner {
        ModifyMultipleOrdersBatchOrdersParameterInner {
            order_id: None,
            orig_client_order_id: None,
            symbol: None,
            side: None,
            quantity: None,
            price: None,
            price_match: None,
            recv_window: None,
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
