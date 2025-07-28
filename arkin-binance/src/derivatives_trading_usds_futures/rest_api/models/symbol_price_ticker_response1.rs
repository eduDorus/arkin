#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct SymbolPriceTickerResponse1 {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "price", skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
}

impl SymbolPriceTickerResponse1 {
    #[must_use]
    pub fn new() -> SymbolPriceTickerResponse1 {
        SymbolPriceTickerResponse1 {
            symbol: None,
            price: None,
            time: None,
        }
    }
}
