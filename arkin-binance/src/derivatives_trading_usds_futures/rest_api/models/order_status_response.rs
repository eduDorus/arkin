#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrderStatusResponse {
    #[serde(rename = "orderId", skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    #[serde(rename = "orderStatus", skip_serializing_if = "Option::is_none")]
    pub order_status: Option<String>,
    #[serde(rename = "fromAsset", skip_serializing_if = "Option::is_none")]
    pub from_asset: Option<String>,
    #[serde(rename = "fromAmount", skip_serializing_if = "Option::is_none")]
    pub from_amount: Option<String>,
    #[serde(rename = "toAsset", skip_serializing_if = "Option::is_none")]
    pub to_asset: Option<String>,
    #[serde(rename = "toAmount", skip_serializing_if = "Option::is_none")]
    pub to_amount: Option<String>,
    #[serde(rename = "ratio", skip_serializing_if = "Option::is_none")]
    pub ratio: Option<String>,
    #[serde(rename = "inverseRatio", skip_serializing_if = "Option::is_none")]
    pub inverse_ratio: Option<String>,
    #[serde(rename = "createTime", skip_serializing_if = "Option::is_none")]
    pub create_time: Option<i64>,
}

impl OrderStatusResponse {
    #[must_use]
    pub fn new() -> OrderStatusResponse {
        OrderStatusResponse {
            order_id: None,
            order_status: None,
            from_asset: None,
            from_amount: None,
            to_asset: None,
            to_amount: None,
            ratio: None,
            inverse_ratio: None,
            create_time: None,
        }
    }
}
