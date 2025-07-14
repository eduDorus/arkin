
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MarkPriceResponse2Inner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "markPrice", skip_serializing_if = "Option::is_none")]
    pub mark_price: Option<String>,
    #[serde(rename = "indexPrice", skip_serializing_if = "Option::is_none")]
    pub index_price: Option<String>,
    #[serde(
        rename = "estimatedSettlePrice",
        skip_serializing_if = "Option::is_none"
    )]
    pub estimated_settle_price: Option<String>,
    #[serde(rename = "lastFundingRate", skip_serializing_if = "Option::is_none")]
    pub last_funding_rate: Option<String>,
    #[serde(rename = "interestRate", skip_serializing_if = "Option::is_none")]
    pub interest_rate: Option<String>,
    #[serde(rename = "nextFundingTime", skip_serializing_if = "Option::is_none")]
    pub next_funding_time: Option<i64>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
}

impl MarkPriceResponse2Inner {
    #[must_use]
    pub fn new() -> MarkPriceResponse2Inner {
        MarkPriceResponse2Inner {
            symbol: None,
            mark_price: None,
            index_price: None,
            estimated_settle_price: None,
            last_funding_rate: None,
            interest_rate: None,
            next_funding_time: None,
            time: None,
        }
    }
}
