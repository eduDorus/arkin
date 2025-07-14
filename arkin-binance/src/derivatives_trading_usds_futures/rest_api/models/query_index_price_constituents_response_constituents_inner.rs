
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryIndexPriceConstituentsResponseConstituentsInner {
    #[serde(rename = "exchange", skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "price", skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(rename = "weight", skip_serializing_if = "Option::is_none")]
    pub weight: Option<String>,
}

impl QueryIndexPriceConstituentsResponseConstituentsInner {
    #[must_use]
    pub fn new() -> QueryIndexPriceConstituentsResponseConstituentsInner {
        QueryIndexPriceConstituentsResponseConstituentsInner {
            exchange: None,
            symbol: None,
            price: None,
            weight: None,
        }
    }
}
