
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuturesTradingQuantitativeRulesIndicatorsResponseIndicatorsAccountInner {
    #[serde(rename = "indicator", skip_serializing_if = "Option::is_none")]
    pub indicator: Option<String>,
    #[serde(rename = "value", skip_serializing_if = "Option::is_none")]
    pub value: Option<i64>,
    #[serde(rename = "triggerValue", skip_serializing_if = "Option::is_none")]
    pub trigger_value: Option<i64>,
    #[serde(rename = "plannedRecoverTime", skip_serializing_if = "Option::is_none")]
    pub planned_recover_time: Option<i64>,
    #[serde(rename = "isLocked", skip_serializing_if = "Option::is_none")]
    pub is_locked: Option<bool>,
}

impl FuturesTradingQuantitativeRulesIndicatorsResponseIndicatorsAccountInner {
    #[must_use]
    pub fn new() -> FuturesTradingQuantitativeRulesIndicatorsResponseIndicatorsAccountInner {
        FuturesTradingQuantitativeRulesIndicatorsResponseIndicatorsAccountInner {
            indicator: None,
            value: None,
            trigger_value: None,
            planned_recover_time: None,
            is_locked: None,
        }
    }
}
