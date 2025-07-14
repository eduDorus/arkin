
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountInformationV3Response {
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
    pub assets: Option<Vec<models::AccountInformationV3ResponseAssetsInner>>,
    #[serde(rename = "positions", skip_serializing_if = "Option::is_none")]
    pub positions: Option<Vec<models::AccountInformationV3ResponsePositionsInner>>,
}

impl AccountInformationV3Response {
    #[must_use]
    pub fn new() -> AccountInformationV3Response {
        AccountInformationV3Response {
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
        }
    }
}
