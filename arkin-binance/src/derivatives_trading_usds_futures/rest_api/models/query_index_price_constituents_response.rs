
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryIndexPriceConstituentsResponse {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    #[serde(rename = "constituents", skip_serializing_if = "Option::is_none")]
    pub constituents: Option<Vec<models::QueryIndexPriceConstituentsResponseConstituentsInner>>,
}

impl QueryIndexPriceConstituentsResponse {
    #[must_use]
    pub fn new() -> QueryIndexPriceConstituentsResponse {
        QueryIndexPriceConstituentsResponse {
            symbol: None,
            time: None,
            constituents: None,
        }
    }
}
