
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotionalAndLeverageBracketsResponse1InnerBracketsInner {
    #[serde(rename = "bracket", skip_serializing_if = "Option::is_none")]
    pub bracket: Option<i64>,
    #[serde(rename = "initialLeverage", skip_serializing_if = "Option::is_none")]
    pub initial_leverage: Option<i64>,
    #[serde(rename = "notionalCap", skip_serializing_if = "Option::is_none")]
    pub notional_cap: Option<i64>,
    #[serde(rename = "notionalFloor", skip_serializing_if = "Option::is_none")]
    pub notional_floor: Option<i64>,
    #[serde(rename = "maintMarginRatio", skip_serializing_if = "Option::is_none")]
    pub maint_margin_ratio: Option<rust_decimal::Decimal>,
    #[serde(rename = "cum", skip_serializing_if = "Option::is_none")]
    pub cum: Option<i64>,
}

impl NotionalAndLeverageBracketsResponse1InnerBracketsInner {
    #[must_use]
    pub fn new() -> NotionalAndLeverageBracketsResponse1InnerBracketsInner {
        NotionalAndLeverageBracketsResponse1InnerBracketsInner {
            bracket: None,
            initial_leverage: None,
            notional_cap: None,
            notional_floor: None,
            maint_margin_ratio: None,
            cum: None,
        }
    }
}
