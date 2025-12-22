#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Ticker24hrPriceChangeStatisticsResponse {
    Ticker24hrPriceChangeStatisticsResponse1(Box<models::Ticker24hrPriceChangeStatisticsResponse1>),
    Ticker24hrPriceChangeStatisticsResponse2(Vec<models::Ticker24hrPriceChangeStatisticsResponse2Inner>),
    Other(serde_json::Value),
}

impl Default for Ticker24hrPriceChangeStatisticsResponse {
    fn default() -> Self {
        Self::Ticker24hrPriceChangeStatisticsResponse1(Default::default())
    }
}
