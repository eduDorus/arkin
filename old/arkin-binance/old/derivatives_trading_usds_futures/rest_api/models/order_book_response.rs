#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrderBookResponse {
    #[serde(rename = "lastUpdateId", skip_serializing_if = "Option::is_none")]
    pub last_update_id: Option<i64>,
    #[serde(rename = "E", skip_serializing_if = "Option::is_none")]
    pub e_uppercase: Option<i64>,
    #[serde(rename = "T", skip_serializing_if = "Option::is_none")]
    pub t_uppercase: Option<i64>,
    #[serde(rename = "bids", skip_serializing_if = "Option::is_none")]
    pub bids: Option<Vec<Vec<String>>>,
    #[serde(rename = "asks", skip_serializing_if = "Option::is_none")]
    pub asks: Option<Vec<Vec<String>>>,
}

impl OrderBookResponse {
    #[must_use]
    pub fn new() -> OrderBookResponse {
        OrderBookResponse {
            last_update_id: None,
            e_uppercase: None,
            t_uppercase: None,
            bids: None,
            asks: None,
        }
    }
}
