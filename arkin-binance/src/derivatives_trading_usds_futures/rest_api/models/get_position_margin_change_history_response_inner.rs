#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetPositionMarginChangeHistoryResponseInner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<i64>,
    #[serde(rename = "deltaType", skip_serializing_if = "Option::is_none")]
    pub delta_type: Option<String>,
    #[serde(rename = "amount", skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    #[serde(rename = "asset", skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    #[serde(rename = "positionSide", skip_serializing_if = "Option::is_none")]
    pub position_side: Option<String>,
}

impl GetPositionMarginChangeHistoryResponseInner {
    #[must_use]
    pub fn new() -> GetPositionMarginChangeHistoryResponseInner {
        GetPositionMarginChangeHistoryResponseInner {
            symbol: None,
            r#type: None,
            delta_type: None,
            amount: None,
            asset: None,
            time: None,
            position_side: None,
        }
    }
}
