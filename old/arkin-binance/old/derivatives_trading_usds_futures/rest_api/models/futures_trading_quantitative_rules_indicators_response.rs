#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuturesTradingQuantitativeRulesIndicatorsResponse {
    #[serde(rename = "indicators", skip_serializing_if = "Option::is_none")]
    pub indicators: Option<Box<models::FuturesTradingQuantitativeRulesIndicatorsResponseIndicators>>,
    #[serde(rename = "updateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<i64>,
}

impl FuturesTradingQuantitativeRulesIndicatorsResponse {
    #[must_use]
    pub fn new() -> FuturesTradingQuantitativeRulesIndicatorsResponse {
        FuturesTradingQuantitativeRulesIndicatorsResponse {
            indicators: None,
            update_time: None,
        }
    }
}
