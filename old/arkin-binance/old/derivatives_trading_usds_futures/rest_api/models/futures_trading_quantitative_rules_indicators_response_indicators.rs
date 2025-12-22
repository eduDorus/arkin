#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuturesTradingQuantitativeRulesIndicatorsResponseIndicators {
    #[serde(rename = "BTCUSDT", skip_serializing_if = "Option::is_none")]
    pub btcusdt: Option<Vec<models::FuturesTradingQuantitativeRulesIndicatorsResponseIndicatorsBtcusdtInner>>,
    #[serde(rename = "ETHUSDT", skip_serializing_if = "Option::is_none")]
    pub ethusdt: Option<Vec<models::FuturesTradingQuantitativeRulesIndicatorsResponseIndicatorsBtcusdtInner>>,
    #[serde(rename = "ACCOUNT", skip_serializing_if = "Option::is_none")]
    pub account: Option<Vec<models::FuturesTradingQuantitativeRulesIndicatorsResponseIndicatorsAccountInner>>,
}

impl FuturesTradingQuantitativeRulesIndicatorsResponseIndicators {
    #[must_use]
    pub fn new() -> FuturesTradingQuantitativeRulesIndicatorsResponseIndicators {
        FuturesTradingQuantitativeRulesIndicatorsResponseIndicators {
            btcusdt: None,
            ethusdt: None,
            account: None,
        }
    }
}
