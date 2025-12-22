#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SymbolOrderBookTickerResponse {
    SymbolOrderBookTickerResponse1(Box<models::SymbolOrderBookTickerResponse1>),
    SymbolOrderBookTickerResponse2(Vec<models::SymbolOrderBookTickerResponse2Inner>),
    Other(serde_json::Value),
}

impl Default for SymbolOrderBookTickerResponse {
    fn default() -> Self {
        Self::SymbolOrderBookTickerResponse1(Default::default())
    }
}
