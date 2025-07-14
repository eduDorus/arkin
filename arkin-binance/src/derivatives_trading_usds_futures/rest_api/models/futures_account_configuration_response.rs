
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct FuturesAccountConfigurationResponse {
    #[serde(rename = "feeTier", skip_serializing_if = "Option::is_none")]
    pub fee_tier: Option<i64>,
    #[serde(rename = "canTrade", skip_serializing_if = "Option::is_none")]
    pub can_trade: Option<bool>,
    #[serde(rename = "canDeposit", skip_serializing_if = "Option::is_none")]
    pub can_deposit: Option<bool>,
    #[serde(rename = "canWithdraw", skip_serializing_if = "Option::is_none")]
    pub can_withdraw: Option<bool>,
    #[serde(rename = "dualSidePosition", skip_serializing_if = "Option::is_none")]
    pub dual_side_position: Option<bool>,
    #[serde(rename = "updateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<i64>,
    #[serde(rename = "multiAssetsMargin", skip_serializing_if = "Option::is_none")]
    pub multi_assets_margin: Option<bool>,
    #[serde(rename = "tradeGroupId", skip_serializing_if = "Option::is_none")]
    pub trade_group_id: Option<i64>,
}

impl FuturesAccountConfigurationResponse {
    #[must_use]
    pub fn new() -> FuturesAccountConfigurationResponse {
        FuturesAccountConfigurationResponse {
            fee_tier: None,
            can_trade: None,
            can_deposit: None,
            can_withdraw: None,
            dual_side_position: None,
            update_time: None,
            multi_assets_margin: None,
            trade_group_id: None,
        }
    }
}
