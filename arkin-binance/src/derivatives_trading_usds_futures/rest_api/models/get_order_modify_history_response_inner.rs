#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetOrderModifyHistoryResponseInner {
    #[serde(rename = "amendmentId", skip_serializing_if = "Option::is_none")]
    pub amendment_id: Option<i64>,
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "pair", skip_serializing_if = "Option::is_none")]
    pub pair: Option<String>,
    #[serde(rename = "orderId", skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    #[serde(rename = "clientOrderId", skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    #[serde(rename = "amendment", skip_serializing_if = "Option::is_none")]
    pub amendment: Option<Box<models::GetOrderModifyHistoryResponseInnerAmendment>>,
}

impl GetOrderModifyHistoryResponseInner {
    #[must_use]
    pub fn new() -> GetOrderModifyHistoryResponseInner {
        GetOrderModifyHistoryResponseInner {
            amendment_id: None,
            symbol: None,
            pair: None,
            order_id: None,
            client_order_id: None,
            time: None,
            amendment: None,
        }
    }
}
