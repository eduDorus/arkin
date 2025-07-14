
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct AccountTradeListResponseInner {
    #[serde(rename = "buyer", skip_serializing_if = "Option::is_none")]
    pub buyer: Option<bool>,
    #[serde(rename = "commission", skip_serializing_if = "Option::is_none")]
    pub commission: Option<String>,
    #[serde(rename = "commissionAsset", skip_serializing_if = "Option::is_none")]
    pub commission_asset: Option<String>,
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(rename = "maker", skip_serializing_if = "Option::is_none")]
    pub maker: Option<bool>,
    #[serde(rename = "orderId", skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    #[serde(rename = "price", skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(rename = "qty", skip_serializing_if = "Option::is_none")]
    pub qty: Option<String>,
    #[serde(rename = "quoteQty", skip_serializing_if = "Option::is_none")]
    pub quote_qty: Option<String>,
    #[serde(rename = "realizedPnl", skip_serializing_if = "Option::is_none")]
    pub realized_pnl: Option<String>,
    #[serde(rename = "side", skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    #[serde(rename = "positionSide", skip_serializing_if = "Option::is_none")]
    pub position_side: Option<String>,
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
}

impl AccountTradeListResponseInner {
    #[must_use]
    pub fn new() -> AccountTradeListResponseInner {
        AccountTradeListResponseInner {
            buyer: None,
            commission: None,
            commission_asset: None,
            id: None,
            maker: None,
            order_id: None,
            price: None,
            qty: None,
            quote_qty: None,
            realized_pnl: None,
            side: None,
            position_side: None,
            symbol: None,
            time: None,
        }
    }
}
