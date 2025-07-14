
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotionalAndLeverageBracketsResponse2 {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "notionalCoef", skip_serializing_if = "Option::is_none")]
    pub notional_coef: Option<rust_decimal::Decimal>,
    #[serde(rename = "brackets", skip_serializing_if = "Option::is_none")]
    pub brackets: Option<Vec<models::NotionalAndLeverageBracketsResponse1InnerBracketsInner>>,
}

impl NotionalAndLeverageBracketsResponse2 {
    #[must_use]
    pub fn new() -> NotionalAndLeverageBracketsResponse2 {
        NotionalAndLeverageBracketsResponse2 {
            symbol: None,
            notional_coef: None,
            brackets: None,
        }
    }
}
