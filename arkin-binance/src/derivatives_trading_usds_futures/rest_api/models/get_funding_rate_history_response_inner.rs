
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetFundingRateHistoryResponseInner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "fundingRate", skip_serializing_if = "Option::is_none")]
    pub funding_rate: Option<String>,
    #[serde(rename = "fundingTime", skip_serializing_if = "Option::is_none")]
    pub funding_time: Option<i64>,
    #[serde(rename = "markPrice", skip_serializing_if = "Option::is_none")]
    pub mark_price: Option<String>,
}

impl GetFundingRateHistoryResponseInner {
    #[must_use]
    pub fn new() -> GetFundingRateHistoryResponseInner {
        GetFundingRateHistoryResponseInner {
            symbol: None,
            funding_rate: None,
            funding_time: None,
            mark_price: None,
        }
    }
}
