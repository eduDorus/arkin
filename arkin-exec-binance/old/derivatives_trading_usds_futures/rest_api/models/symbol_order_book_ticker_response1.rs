#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct SymbolOrderBookTickerResponse1 {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "bidPrice", skip_serializing_if = "Option::is_none")]
    pub bid_price: Option<String>,
    #[serde(rename = "bidQty", skip_serializing_if = "Option::is_none")]
    pub bid_qty: Option<String>,
    #[serde(rename = "askPrice", skip_serializing_if = "Option::is_none")]
    pub ask_price: Option<String>,
    #[serde(rename = "askQty", skip_serializing_if = "Option::is_none")]
    pub ask_qty: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
}

impl SymbolOrderBookTickerResponse1 {
    #[must_use]
    pub fn new() -> SymbolOrderBookTickerResponse1 {
        SymbolOrderBookTickerResponse1 {
            symbol: None,
            bid_price: None,
            bid_qty: None,
            ask_price: None,
            ask_qty: None,
            time: None,
        }
    }
}
