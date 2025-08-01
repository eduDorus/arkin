#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct OpenInterestStatisticsResponseInner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "sumOpenInterest", skip_serializing_if = "Option::is_none")]
    pub sum_open_interest: Option<String>,
    #[serde(
        rename = "sumOpenInterestValue",
        skip_serializing_if = "Option::is_none"
    )]
    pub sum_open_interest_value: Option<String>,
    #[serde(
        rename = "CMCCirculatingSupply",
        skip_serializing_if = "Option::is_none"
    )]
    pub cmc_circulating_supply: Option<String>,
    #[serde(rename = "timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl OpenInterestStatisticsResponseInner {
    #[must_use]
    pub fn new() -> OpenInterestStatisticsResponseInner {
        OpenInterestStatisticsResponseInner {
            symbol: None,
            sum_open_interest: None,
            sum_open_interest_value: None,
            cmc_circulating_supply: None,
            timestamp: None,
        }
    }
}
