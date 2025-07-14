
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpenInterestResponse {
    #[serde(rename = "openInterest", skip_serializing_if = "Option::is_none")]
    pub open_interest: Option<String>,
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
}

impl OpenInterestResponse {
    #[must_use]
    pub fn new() -> OpenInterestResponse {
        OpenInterestResponse {
            open_interest: None,
            symbol: None,
            time: None,
        }
    }
}
