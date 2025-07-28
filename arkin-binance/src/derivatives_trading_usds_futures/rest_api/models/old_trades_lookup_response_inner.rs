#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct OldTradesLookupResponseInner {
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(rename = "price", skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(rename = "qty", skip_serializing_if = "Option::is_none")]
    pub qty: Option<String>,
    #[serde(rename = "quoteQty", skip_serializing_if = "Option::is_none")]
    pub quote_qty: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    #[serde(rename = "isBuyerMaker", skip_serializing_if = "Option::is_none")]
    pub is_buyer_maker: Option<bool>,
}

impl OldTradesLookupResponseInner {
    #[must_use]
    pub fn new() -> OldTradesLookupResponseInner {
        OldTradesLookupResponseInner {
            id: None,
            price: None,
            qty: None,
            quote_qty: None,
            time: None,
            is_buyer_maker: None,
        }
    }
}
