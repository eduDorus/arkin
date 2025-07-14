
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionInformationV2ResponseInner {
    #[serde(rename = "entryPrice", skip_serializing_if = "Option::is_none")]
    pub entry_price: Option<String>,
    #[serde(rename = "breakEvenPrice", skip_serializing_if = "Option::is_none")]
    pub break_even_price: Option<String>,
    #[serde(rename = "marginType", skip_serializing_if = "Option::is_none")]
    pub margin_type: Option<String>,
    #[serde(rename = "isAutoAddMargin", skip_serializing_if = "Option::is_none")]
    pub is_auto_add_margin: Option<String>,
    #[serde(rename = "isolatedMargin", skip_serializing_if = "Option::is_none")]
    pub isolated_margin: Option<String>,
    #[serde(rename = "leverage", skip_serializing_if = "Option::is_none")]
    pub leverage: Option<String>,
    #[serde(rename = "liquidationPrice", skip_serializing_if = "Option::is_none")]
    pub liquidation_price: Option<String>,
    #[serde(rename = "markPrice", skip_serializing_if = "Option::is_none")]
    pub mark_price: Option<String>,
    #[serde(rename = "maxNotionalValue", skip_serializing_if = "Option::is_none")]
    pub max_notional_value: Option<String>,
    #[serde(rename = "positionAmt", skip_serializing_if = "Option::is_none")]
    pub position_amt: Option<String>,
    #[serde(rename = "notional", skip_serializing_if = "Option::is_none")]
    pub notional: Option<String>,
    #[serde(rename = "isolatedWallet", skip_serializing_if = "Option::is_none")]
    pub isolated_wallet: Option<String>,
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "unRealizedProfit", skip_serializing_if = "Option::is_none")]
    pub un_realized_profit: Option<String>,
    #[serde(rename = "positionSide", skip_serializing_if = "Option::is_none")]
    pub position_side: Option<String>,
    #[serde(rename = "updateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<i64>,
}

impl PositionInformationV2ResponseInner {
    #[must_use]
    pub fn new() -> PositionInformationV2ResponseInner {
        PositionInformationV2ResponseInner {
            entry_price: None,
            break_even_price: None,
            margin_type: None,
            is_auto_add_margin: None,
            isolated_margin: None,
            leverage: None,
            liquidation_price: None,
            mark_price: None,
            max_notional_value: None,
            position_amt: None,
            notional: None,
            isolated_wallet: None,
            symbol: None,
            un_realized_profit: None,
            position_side: None,
            update_time: None,
        }
    }
}
