#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct NotionalAndLeverageBracketsResponse1Inner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "notionalCoef", skip_serializing_if = "Option::is_none")]
    pub notional_coef: Option<rust_decimal::Decimal>,
    #[serde(rename = "brackets", skip_serializing_if = "Option::is_none")]
    pub brackets: Option<Vec<models::NotionalAndLeverageBracketsResponse1InnerBracketsInner>>,
}

impl NotionalAndLeverageBracketsResponse1Inner {
    #[must_use]
    pub fn new() -> NotionalAndLeverageBracketsResponse1Inner {
        NotionalAndLeverageBracketsResponse1Inner {
            symbol: None,
            notional_coef: None,
            brackets: None,
        }
    }
}
