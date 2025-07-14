
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SymbolPriceTickerV2Response {
    SymbolPriceTickerV2Response1(Box<models::SymbolPriceTickerV2Response1>),
    SymbolPriceTickerV2Response2(Vec<models::SymbolPriceTickerV2Response2Inner>),
    Other(serde_json::Value),
}

impl Default for SymbolPriceTickerV2Response {
    fn default() -> Self {
        Self::SymbolPriceTickerV2Response1(Default::default())
    }
}
