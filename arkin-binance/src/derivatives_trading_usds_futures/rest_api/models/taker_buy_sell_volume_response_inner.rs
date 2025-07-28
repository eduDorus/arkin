#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TakerBuySellVolumeResponseInner {
    #[serde(rename = "buySellRatio", skip_serializing_if = "Option::is_none")]
    pub buy_sell_ratio: Option<String>,
    #[serde(rename = "buyVol", skip_serializing_if = "Option::is_none")]
    pub buy_vol: Option<String>,
    #[serde(rename = "sellVol", skip_serializing_if = "Option::is_none")]
    pub sell_vol: Option<String>,
    #[serde(rename = "timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

impl TakerBuySellVolumeResponseInner {
    #[must_use]
    pub fn new() -> TakerBuySellVolumeResponseInner {
        TakerBuySellVolumeResponseInner {
            buy_sell_ratio: None,
            buy_vol: None,
            sell_vol: None,
            timestamp: None,
        }
    }
}
