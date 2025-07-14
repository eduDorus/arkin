
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserCommissionRateResponse {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(
        rename = "makerCommissionRate",
        skip_serializing_if = "Option::is_none"
    )]
    pub maker_commission_rate: Option<String>,
    #[serde(
        rename = "takerCommissionRate",
        skip_serializing_if = "Option::is_none"
    )]
    pub taker_commission_rate: Option<String>,
}

impl UserCommissionRateResponse {
    #[must_use]
    pub fn new() -> UserCommissionRateResponse {
        UserCommissionRateResponse {
            symbol: None,
            maker_commission_rate: None,
            taker_commission_rate: None,
        }
    }
}
