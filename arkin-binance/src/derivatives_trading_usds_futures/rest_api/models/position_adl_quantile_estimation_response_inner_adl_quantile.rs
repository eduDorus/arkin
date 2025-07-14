
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionAdlQuantileEstimationResponseInnerAdlQuantile {
    #[serde(rename = "LONG", skip_serializing_if = "Option::is_none")]
    pub long: Option<i64>,
    #[serde(rename = "SHORT", skip_serializing_if = "Option::is_none")]
    pub short: Option<i64>,
    #[serde(rename = "HEDGE", skip_serializing_if = "Option::is_none")]
    pub hedge: Option<i64>,
    #[serde(rename = "BOTH", skip_serializing_if = "Option::is_none")]
    pub both: Option<i64>,
}

impl PositionAdlQuantileEstimationResponseInnerAdlQuantile {
    #[must_use]
    pub fn new() -> PositionAdlQuantileEstimationResponseInnerAdlQuantile {
        PositionAdlQuantileEstimationResponseInnerAdlQuantile {
            long: None,
            short: None,
            hedge: None,
            both: None,
        }
    }
}
