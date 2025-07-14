
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetFundingRateInfoResponseInner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(
        rename = "adjustedFundingRateCap",
        skip_serializing_if = "Option::is_none"
    )]
    pub adjusted_funding_rate_cap: Option<String>,
    #[serde(
        rename = "adjustedFundingRateFloor",
        skip_serializing_if = "Option::is_none"
    )]
    pub adjusted_funding_rate_floor: Option<String>,
    #[serde(
        rename = "fundingIntervalHours",
        skip_serializing_if = "Option::is_none"
    )]
    pub funding_interval_hours: Option<i64>,
    #[serde(rename = "disclaimer", skip_serializing_if = "Option::is_none")]
    pub disclaimer: Option<bool>,
}

impl GetFundingRateInfoResponseInner {
    #[must_use]
    pub fn new() -> GetFundingRateInfoResponseInner {
        GetFundingRateInfoResponseInner {
            symbol: None,
            adjusted_funding_rate_cap: None,
            adjusted_funding_rate_floor: None,
            funding_interval_hours: None,
            disclaimer: None,
        }
    }
}
