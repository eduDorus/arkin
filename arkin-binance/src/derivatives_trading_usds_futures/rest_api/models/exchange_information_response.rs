#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExchangeInformationResponse {
    #[serde(rename = "exchangeFilters", skip_serializing_if = "Option::is_none")]
    pub exchange_filters: Option<Vec<String>>,
    #[serde(rename = "rateLimits", skip_serializing_if = "Option::is_none")]
    pub rate_limits: Option<Vec<models::ExchangeInformationResponseRateLimitsInner>>,
    #[serde(rename = "serverTime", skip_serializing_if = "Option::is_none")]
    pub server_time: Option<i64>,
    #[serde(rename = "assets", skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<models::ExchangeInformationResponseAssetsInner>>,
    #[serde(rename = "symbols", skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<models::ExchangeInformationResponseSymbolsInner>>,
    #[serde(rename = "timezone", skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

impl ExchangeInformationResponse {
    #[must_use]
    pub fn new() -> ExchangeInformationResponse {
        ExchangeInformationResponse {
            exchange_filters: None,
            rate_limits: None,
            server_time: None,
            assets: None,
            symbols: None,
            timezone: None,
        }
    }
}
