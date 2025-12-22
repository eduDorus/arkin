#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompressedAggregateTradesListResponseInner {
    #[serde(rename = "a", skip_serializing_if = "Option::is_none")]
    pub a: Option<i64>,
    #[serde(rename = "p", skip_serializing_if = "Option::is_none")]
    pub p: Option<String>,
    #[serde(rename = "q", skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(rename = "f", skip_serializing_if = "Option::is_none")]
    pub f: Option<i64>,
    #[serde(rename = "l", skip_serializing_if = "Option::is_none")]
    pub l: Option<i64>,
    #[serde(rename = "T", skip_serializing_if = "Option::is_none")]
    pub t_uppercase: Option<i64>,
    #[serde(rename = "m", skip_serializing_if = "Option::is_none")]
    pub m: Option<bool>,
}

impl CompressedAggregateTradesListResponseInner {
    #[must_use]
    pub fn new() -> CompressedAggregateTradesListResponseInner {
        CompressedAggregateTradesListResponseInner {
            a: None,
            p: None,
            q: None,
            f: None,
            l: None,
            t_uppercase: None,
            m: None,
        }
    }
}
