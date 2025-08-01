#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ticker24hrPriceChangeStatisticsResponse1 {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "priceChange", skip_serializing_if = "Option::is_none")]
    pub price_change: Option<String>,
    #[serde(rename = "priceChangePercent", skip_serializing_if = "Option::is_none")]
    pub price_change_percent: Option<String>,
    #[serde(rename = "weightedAvgPrice", skip_serializing_if = "Option::is_none")]
    pub weighted_avg_price: Option<String>,
    #[serde(rename = "lastPrice", skip_serializing_if = "Option::is_none")]
    pub last_price: Option<String>,
    #[serde(rename = "lastQty", skip_serializing_if = "Option::is_none")]
    pub last_qty: Option<String>,
    #[serde(rename = "openPrice", skip_serializing_if = "Option::is_none")]
    pub open_price: Option<String>,
    #[serde(rename = "highPrice", skip_serializing_if = "Option::is_none")]
    pub high_price: Option<String>,
    #[serde(rename = "lowPrice", skip_serializing_if = "Option::is_none")]
    pub low_price: Option<String>,
    #[serde(rename = "volume", skip_serializing_if = "Option::is_none")]
    pub volume: Option<String>,
    #[serde(rename = "quoteVolume", skip_serializing_if = "Option::is_none")]
    pub quote_volume: Option<String>,
    #[serde(rename = "openTime", skip_serializing_if = "Option::is_none")]
    pub open_time: Option<i64>,
    #[serde(rename = "closeTime", skip_serializing_if = "Option::is_none")]
    pub close_time: Option<i64>,
    #[serde(rename = "firstId", skip_serializing_if = "Option::is_none")]
    pub first_id: Option<i64>,
    #[serde(rename = "lastId", skip_serializing_if = "Option::is_none")]
    pub last_id: Option<i64>,
    #[serde(rename = "count", skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
}

impl Ticker24hrPriceChangeStatisticsResponse1 {
    #[must_use]
    pub fn new() -> Ticker24hrPriceChangeStatisticsResponse1 {
        Ticker24hrPriceChangeStatisticsResponse1 {
            symbol: None,
            price_change: None,
            price_change_percent: None,
            weighted_avg_price: None,
            last_price: None,
            last_qty: None,
            open_price: None,
            high_price: None,
            low_price: None,
            volume: None,
            quote_volume: None,
            open_time: None,
            close_time: None,
            first_id: None,
            last_id: None,
            count: None,
        }
    }
}
