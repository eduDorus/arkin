#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuturesAccountBalanceV2ResponseInner {
    #[serde(rename = "accountAlias", skip_serializing_if = "Option::is_none")]
    pub account_alias: Option<String>,
    #[serde(rename = "asset", skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    #[serde(rename = "balance", skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,
    #[serde(rename = "crossWalletBalance", skip_serializing_if = "Option::is_none")]
    pub cross_wallet_balance: Option<String>,
    #[serde(rename = "crossUnPnl", skip_serializing_if = "Option::is_none")]
    pub cross_un_pnl: Option<String>,
    #[serde(rename = "availableBalance", skip_serializing_if = "Option::is_none")]
    pub available_balance: Option<String>,
    #[serde(rename = "maxWithdrawAmount", skip_serializing_if = "Option::is_none")]
    pub max_withdraw_amount: Option<String>,
    #[serde(rename = "marginAvailable", skip_serializing_if = "Option::is_none")]
    pub margin_available: Option<bool>,
    #[serde(rename = "updateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<i64>,
}

impl FuturesAccountBalanceV2ResponseInner {
    #[must_use]
    pub fn new() -> FuturesAccountBalanceV2ResponseInner {
        FuturesAccountBalanceV2ResponseInner {
            account_alias: None,
            asset: None,
            balance: None,
            cross_wallet_balance: None,
            cross_un_pnl: None,
            available_balance: None,
            max_withdraw_amount: None,
            margin_available: None,
            update_time: None,
        }
    }
}
