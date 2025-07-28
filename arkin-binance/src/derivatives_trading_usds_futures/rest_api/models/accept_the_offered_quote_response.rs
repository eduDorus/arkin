#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct AcceptTheOfferedQuoteResponse {
    #[serde(rename = "orderId", skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    #[serde(rename = "createTime", skip_serializing_if = "Option::is_none")]
    pub create_time: Option<i64>,
    #[serde(rename = "orderStatus", skip_serializing_if = "Option::is_none")]
    pub order_status: Option<String>,
}

impl AcceptTheOfferedQuoteResponse {
    #[must_use]
    pub fn new() -> AcceptTheOfferedQuoteResponse {
        AcceptTheOfferedQuoteResponse {
            order_id: None,
            create_time: None,
            order_status: None,
        }
    }
}
