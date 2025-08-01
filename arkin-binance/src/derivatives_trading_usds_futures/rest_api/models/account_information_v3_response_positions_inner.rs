#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountInformationV3ResponsePositionsInner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "positionSide", skip_serializing_if = "Option::is_none")]
    pub position_side: Option<String>,
    #[serde(rename = "positionAmt", skip_serializing_if = "Option::is_none")]
    pub position_amt: Option<String>,
    #[serde(rename = "unrealizedProfit", skip_serializing_if = "Option::is_none")]
    pub unrealized_profit: Option<String>,
    #[serde(rename = "isolatedMargin", skip_serializing_if = "Option::is_none")]
    pub isolated_margin: Option<String>,
    #[serde(rename = "notional", skip_serializing_if = "Option::is_none")]
    pub notional: Option<String>,
    #[serde(rename = "isolatedWallet", skip_serializing_if = "Option::is_none")]
    pub isolated_wallet: Option<String>,
    #[serde(rename = "initialMargin", skip_serializing_if = "Option::is_none")]
    pub initial_margin: Option<String>,
    #[serde(rename = "maintMargin", skip_serializing_if = "Option::is_none")]
    pub maint_margin: Option<String>,
    #[serde(rename = "updateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<i64>,
}

impl AccountInformationV3ResponsePositionsInner {
    #[must_use]
    pub fn new() -> AccountInformationV3ResponsePositionsInner {
        AccountInformationV3ResponsePositionsInner {
            symbol: None,
            position_side: None,
            position_amt: None,
            unrealized_profit: None,
            isolated_margin: None,
            notional: None,
            isolated_wallet: None,
            initial_margin: None,
            maint_margin: None,
            update_time: None,
        }
    }
}
