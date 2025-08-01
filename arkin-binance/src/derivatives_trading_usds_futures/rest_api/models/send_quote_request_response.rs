#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct SendQuoteRequestResponse {
    #[serde(rename = "quoteId", skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<String>,
    #[serde(rename = "ratio", skip_serializing_if = "Option::is_none")]
    pub ratio: Option<String>,
    #[serde(rename = "inverseRatio", skip_serializing_if = "Option::is_none")]
    pub inverse_ratio: Option<String>,
    #[serde(rename = "validTimestamp", skip_serializing_if = "Option::is_none")]
    pub valid_timestamp: Option<i64>,
    #[serde(rename = "toAmount", skip_serializing_if = "Option::is_none")]
    pub to_amount: Option<String>,
    #[serde(rename = "fromAmount", skip_serializing_if = "Option::is_none")]
    pub from_amount: Option<String>,
}

impl SendQuoteRequestResponse {
    #[must_use]
    pub fn new() -> SendQuoteRequestResponse {
        SendQuoteRequestResponse {
            quote_id: None,
            ratio: None,
            inverse_ratio: None,
            valid_timestamp: None,
            to_amount: None,
            from_amount: None,
        }
    }
}
