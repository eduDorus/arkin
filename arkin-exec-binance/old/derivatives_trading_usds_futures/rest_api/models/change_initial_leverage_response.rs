#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChangeInitialLeverageResponse {
    #[serde(rename = "leverage", skip_serializing_if = "Option::is_none")]
    pub leverage: Option<i64>,
    #[serde(rename = "maxNotionalValue", skip_serializing_if = "Option::is_none")]
    pub max_notional_value: Option<String>,
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
}

impl ChangeInitialLeverageResponse {
    #[must_use]
    pub fn new() -> ChangeInitialLeverageResponse {
        ChangeInitialLeverageResponse {
            leverage: None,
            max_notional_value: None,
            symbol: None,
        }
    }
}
