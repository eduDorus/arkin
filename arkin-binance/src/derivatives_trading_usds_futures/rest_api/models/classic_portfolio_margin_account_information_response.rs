
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ClassicPortfolioMarginAccountInformationResponse {
    #[serde(
        rename = "maxWithdrawAmountUSD",
        skip_serializing_if = "Option::is_none"
    )]
    pub max_withdraw_amount_usd: Option<String>,
    #[serde(rename = "asset", skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    #[serde(rename = "maxWithdrawAmount", skip_serializing_if = "Option::is_none")]
    pub max_withdraw_amount: Option<String>,
}

impl ClassicPortfolioMarginAccountInformationResponse {
    #[must_use]
    pub fn new() -> ClassicPortfolioMarginAccountInformationResponse {
        ClassicPortfolioMarginAccountInformationResponse {
            max_withdraw_amount_usd: None,
            asset: None,
            max_withdraw_amount: None,
        }
    }
}
