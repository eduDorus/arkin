
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct SymbolPriceTickerV2Response2Inner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "price", skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
}

impl SymbolPriceTickerV2Response2Inner {
    #[must_use]
    pub fn new() -> SymbolPriceTickerV2Response2Inner {
        SymbolPriceTickerV2Response2Inner {
            symbol: None,
            price: None,
            time: None,
        }
    }
}
