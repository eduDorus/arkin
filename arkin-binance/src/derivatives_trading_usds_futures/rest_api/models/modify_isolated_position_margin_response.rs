#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ModifyIsolatedPositionMarginResponse {
    #[serde(rename = "amount", skip_serializing_if = "Option::is_none")]
    pub amount: Option<rust_decimal::Decimal>,
    #[serde(rename = "code", skip_serializing_if = "Option::is_none")]
    pub code: Option<i64>,
    #[serde(rename = "msg", skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<i64>,
}

impl ModifyIsolatedPositionMarginResponse {
    #[must_use]
    pub fn new() -> ModifyIsolatedPositionMarginResponse {
        ModifyIsolatedPositionMarginResponse {
            amount: None,
            code: None,
            msg: None,
            r#type: None,
        }
    }
}
