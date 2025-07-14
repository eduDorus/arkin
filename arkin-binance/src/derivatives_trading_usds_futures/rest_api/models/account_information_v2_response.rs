
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountInformationV2Response {
    #[serde(rename = "feeTier", skip_serializing_if = "Option::is_none")]
    pub fee_tier: Option<i64>,
    #[serde(rename = "feeBurn", skip_serializing_if = "Option::is_none")]
    pub fee_burn: Option<bool>,
    #[serde(rename = "canDeposit", skip_serializing_if = "Option::is_none")]
    pub can_deposit: Option<bool>,
    #[serde(rename = "canWithdraw", skip_serializing_if = "Option::is_none")]
    pub can_withdraw: Option<bool>,
    #[serde(rename = "updateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<i64>,
    #[serde(rename = "multiAssetsMargin", skip_serializing_if = "Option::is_none")]
    pub multi_assets_margin: Option<bool>,
    #[serde(rename = "tradeGroupId", skip_serializing_if = "Option::is_none")]
    pub trade_group_id: Option<i64>,
    #[serde(rename = "totalInitialMargin", skip_serializing_if = "Option::is_none")]
    pub total_initial_margin: Option<String>,
    #[serde(rename = "totalMaintMargin", skip_serializing_if = "Option::is_none")]
    pub total_maint_margin: Option<String>,
    #[serde(rename = "totalWalletBalance", skip_serializing_if = "Option::is_none")]
    pub total_wallet_balance: Option<String>,
    #[serde(
        rename = "totalUnrealizedProfit",
        skip_serializing_if = "Option::is_none"
    )]
    pub total_unrealized_profit: Option<String>,
    #[serde(rename = "totalMarginBalance", skip_serializing_if = "Option::is_none")]
    pub total_margin_balance: Option<String>,
    #[serde(
        rename = "totalPositionInitialMargin",
        skip_serializing_if = "Option::is_none"
    )]
    pub total_position_initial_margin: Option<String>,
    #[serde(
        rename = "totalOpenOrderInitialMargin",
        skip_serializing_if = "Option::is_none"
    )]
    pub total_open_order_initial_margin: Option<String>,
    #[serde(
        rename = "totalCrossWalletBalance",
        skip_serializing_if = "Option::is_none"
    )]
    pub total_cross_wallet_balance: Option<String>,
    #[serde(rename = "totalCrossUnPnl", skip_serializing_if = "Option::is_none")]
    pub total_cross_un_pnl: Option<String>,
    #[serde(rename = "availableBalance", skip_serializing_if = "Option::is_none")]
    pub available_balance: Option<String>,
    #[serde(rename = "maxWithdrawAmount", skip_serializing_if = "Option::is_none")]
    pub max_withdraw_amount: Option<String>,
    #[serde(rename = "assets", skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<models::AccountInformationV2ResponseAssetsInner>>,
    #[serde(rename = "positions", skip_serializing_if = "Option::is_none")]
    pub positions: Option<Vec<models::AccountInformationV2ResponsePositionsInner>>,
    #[serde(rename = "canTrade", skip_serializing_if = "Option::is_none")]
    pub can_trade: Option<bool>,
}

impl AccountInformationV2Response {
    #[must_use]
    pub fn new() -> AccountInformationV2Response {
        AccountInformationV2Response {
            fee_tier: None,
            fee_burn: None,
            can_deposit: None,
            can_withdraw: None,
            update_time: None,
            multi_assets_margin: None,
            trade_group_id: None,
            total_initial_margin: None,
            total_maint_margin: None,
            total_wallet_balance: None,
            total_unrealized_profit: None,
            total_margin_balance: None,
            total_position_initial_margin: None,
            total_open_order_initial_margin: None,
            total_cross_wallet_balance: None,
            total_cross_un_pnl: None,
            available_balance: None,
            max_withdraw_amount: None,
            assets: None,
            positions: None,
            can_trade: None,
        }
    }
}
