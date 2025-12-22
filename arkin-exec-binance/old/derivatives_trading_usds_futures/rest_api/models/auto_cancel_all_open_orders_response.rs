#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct AutoCancelAllOpenOrdersResponse {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "countdownTime", skip_serializing_if = "Option::is_none")]
    pub countdown_time: Option<String>,
}

impl AutoCancelAllOpenOrdersResponse {
    #[must_use]
    pub fn new() -> AutoCancelAllOpenOrdersResponse {
        AutoCancelAllOpenOrdersResponse {
            symbol: None,
            countdown_time: None,
        }
    }
}
