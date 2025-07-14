
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionAdlQuantileEstimationResponseInner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "adlQuantile", skip_serializing_if = "Option::is_none")]
    pub adl_quantile: Option<Box<models::PositionAdlQuantileEstimationResponseInnerAdlQuantile>>,
}

impl PositionAdlQuantileEstimationResponseInner {
    #[must_use]
    pub fn new() -> PositionAdlQuantileEstimationResponseInner {
        PositionAdlQuantileEstimationResponseInner {
            symbol: None,
            adl_quantile: None,
        }
    }
}
