#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryUserRateLimitResponseInner {
    #[serde(rename = "rateLimitType", skip_serializing_if = "Option::is_none")]
    pub rate_limit_type: Option<String>,
    #[serde(rename = "interval", skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    #[serde(rename = "intervalNum", skip_serializing_if = "Option::is_none")]
    pub interval_num: Option<i64>,
    #[serde(rename = "limit", skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

impl QueryUserRateLimitResponseInner {
    #[must_use]
    pub fn new() -> QueryUserRateLimitResponseInner {
        QueryUserRateLimitResponseInner {
            rate_limit_type: None,
            interval: None,
            interval_num: None,
            limit: None,
        }
    }
}
