#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ToggleBnbBurnOnFuturesTradeResponse {
    #[serde(rename = "code", skip_serializing_if = "Option::is_none")]
    pub code: Option<i64>,
    #[serde(rename = "msg", skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
}

impl ToggleBnbBurnOnFuturesTradeResponse {
    #[must_use]
    pub fn new() -> ToggleBnbBurnOnFuturesTradeResponse {
        ToggleBnbBurnOnFuturesTradeResponse {
            code: None,
            msg: None,
        }
    }
}
