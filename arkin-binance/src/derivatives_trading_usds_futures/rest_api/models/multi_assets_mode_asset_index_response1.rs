
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct MultiAssetsModeAssetIndexResponse1 {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    #[serde(rename = "index", skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,
    #[serde(rename = "bidBuffer", skip_serializing_if = "Option::is_none")]
    pub bid_buffer: Option<String>,
    #[serde(rename = "askBuffer", skip_serializing_if = "Option::is_none")]
    pub ask_buffer: Option<String>,
    #[serde(rename = "bidRate", skip_serializing_if = "Option::is_none")]
    pub bid_rate: Option<String>,
    #[serde(rename = "askRate", skip_serializing_if = "Option::is_none")]
    pub ask_rate: Option<String>,
    #[serde(
        rename = "autoExchangeBidBuffer",
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_exchange_bid_buffer: Option<String>,
    #[serde(
        rename = "autoExchangeAskBuffer",
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_exchange_ask_buffer: Option<String>,
    #[serde(
        rename = "autoExchangeBidRate",
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_exchange_bid_rate: Option<String>,
    #[serde(
        rename = "autoExchangeAskRate",
        skip_serializing_if = "Option::is_none"
    )]
    pub auto_exchange_ask_rate: Option<String>,
}

impl MultiAssetsModeAssetIndexResponse1 {
    #[must_use]
    pub fn new() -> MultiAssetsModeAssetIndexResponse1 {
        MultiAssetsModeAssetIndexResponse1 {
            symbol: None,
            time: None,
            index: None,
            bid_buffer: None,
            ask_buffer: None,
            bid_rate: None,
            ask_rate: None,
            auto_exchange_bid_buffer: None,
            auto_exchange_ask_buffer: None,
            auto_exchange_bid_rate: None,
            auto_exchange_ask_rate: None,
        }
    }
}
