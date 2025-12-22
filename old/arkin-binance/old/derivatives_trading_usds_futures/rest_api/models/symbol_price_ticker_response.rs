#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SymbolPriceTickerResponse {
    SymbolPriceTickerResponse1(Box<models::SymbolPriceTickerResponse1>),
    SymbolPriceTickerResponse2(Vec<models::SymbolPriceTickerV2Response2Inner>),
    Other(serde_json::Value),
}

impl Default for SymbolPriceTickerResponse {
    fn default() -> Self {
        Self::SymbolPriceTickerResponse1(Default::default())
    }
}
