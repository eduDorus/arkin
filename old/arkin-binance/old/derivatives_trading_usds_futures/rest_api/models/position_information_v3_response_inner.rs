#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionInformationV3ResponseInner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "positionSide", skip_serializing_if = "Option::is_none")]
    pub position_side: Option<String>,
    #[serde(rename = "positionAmt", skip_serializing_if = "Option::is_none")]
    pub position_amt: Option<String>,
    #[serde(rename = "entryPrice", skip_serializing_if = "Option::is_none")]
    pub entry_price: Option<String>,
    #[serde(rename = "breakEvenPrice", skip_serializing_if = "Option::is_none")]
    pub break_even_price: Option<String>,
    #[serde(rename = "markPrice", skip_serializing_if = "Option::is_none")]
    pub mark_price: Option<String>,
    #[serde(rename = "unRealizedProfit", skip_serializing_if = "Option::is_none")]
    pub un_realized_profit: Option<String>,
    #[serde(rename = "liquidationPrice", skip_serializing_if = "Option::is_none")]
    pub liquidation_price: Option<String>,
    #[serde(rename = "isolatedMargin", skip_serializing_if = "Option::is_none")]
    pub isolated_margin: Option<String>,
    #[serde(rename = "notional", skip_serializing_if = "Option::is_none")]
    pub notional: Option<String>,
    #[serde(rename = "marginAsset", skip_serializing_if = "Option::is_none")]
    pub margin_asset: Option<String>,
    #[serde(rename = "isolatedWallet", skip_serializing_if = "Option::is_none")]
    pub isolated_wallet: Option<String>,
    #[serde(rename = "initialMargin", skip_serializing_if = "Option::is_none")]
    pub initial_margin: Option<String>,
    #[serde(rename = "maintMargin", skip_serializing_if = "Option::is_none")]
    pub maint_margin: Option<String>,
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
    #[serde(rename = "adl", skip_serializing_if = "Option::is_none")]
    pub adl: Option<i64>,
    #[serde(rename = "bidNotional", skip_serializing_if = "Option::is_none")]
    pub bid_notional: Option<String>,
    #[serde(rename = "askNotional", skip_serializing_if = "Option::is_none")]
    pub ask_notional: Option<String>,
    #[serde(rename = "updateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<i64>,
}

impl PositionInformationV3ResponseInner {
    #[must_use]
    pub fn new() -> PositionInformationV3ResponseInner {
        PositionInformationV3ResponseInner {
            symbol: None,
            position_side: None,
            position_amt: None,
            entry_price: None,
            break_even_price: None,
            mark_price: None,
            un_realized_profit: None,
            liquidation_price: None,
            isolated_margin: None,
            notional: None,
            margin_asset: None,
            isolated_wallet: None,
            initial_margin: None,
            maint_margin: None,
            position_initial_margin: None,
            open_order_initial_margin: None,
            adl: None,
            bid_notional: None,
            ask_notional: None,
            update_time: None,
        }
    }
}
