#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountInformationV2ResponsePositionsInner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "initialMargin", skip_serializing_if = "Option::is_none")]
    pub initial_margin: Option<String>,
    #[serde(rename = "maintMargin", skip_serializing_if = "Option::is_none")]
    pub maint_margin: Option<String>,
    #[serde(rename = "unrealizedProfit", skip_serializing_if = "Option::is_none")]
    pub unrealized_profit: Option<String>,
    #[serde(
        rename = "positionInitialMargin",
        skip_serializing_if = "Option::is_none"
    )]
    pub position_initial_margin: Option<String>,
    #[serde(
        rename = "openOrderInitialMargin",
        skip_serializing_if = "Option::is_none"
    )]
    pub open_order_initial_margin: Option<String>,
    #[serde(rename = "leverage", skip_serializing_if = "Option::is_none")]
    pub leverage: Option<String>,
    #[serde(rename = "isolated", skip_serializing_if = "Option::is_none")]
    pub isolated: Option<bool>,
    #[serde(rename = "entryPrice", skip_serializing_if = "Option::is_none")]
    pub entry_price: Option<String>,
    #[serde(rename = "maxNotional", skip_serializing_if = "Option::is_none")]
    pub max_notional: Option<String>,
    #[serde(rename = "bidNotional", skip_serializing_if = "Option::is_none")]
    pub bid_notional: Option<String>,
    #[serde(rename = "askNotional", skip_serializing_if = "Option::is_none")]
    pub ask_notional: Option<String>,
    #[serde(rename = "positionSide", skip_serializing_if = "Option::is_none")]
    pub position_side: Option<String>,
    #[serde(rename = "positionAmt", skip_serializing_if = "Option::is_none")]
    pub position_amt: Option<String>,
    #[serde(rename = "updateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<i64>,
}

impl AccountInformationV2ResponsePositionsInner {
    #[must_use]
    pub fn new() -> AccountInformationV2ResponsePositionsInner {
        AccountInformationV2ResponsePositionsInner {
            symbol: None,
            initial_margin: None,
            maint_margin: None,
            unrealized_profit: None,
            position_initial_margin: None,
            open_order_initial_margin: None,
            leverage: None,
            isolated: None,
            entry_price: None,
            max_notional: None,
            bid_notional: None,
            ask_notional: None,
            position_side: None,
            position_amt: None,
            update_time: None,
        }
    }
}
