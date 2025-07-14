
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetOrderModifyHistoryResponseInnerAmendment {
    #[serde(rename = "price", skip_serializing_if = "Option::is_none")]
    pub price: Option<Box<models::GetOrderModifyHistoryResponseInnerAmendmentPrice>>,
    #[serde(rename = "origQty", skip_serializing_if = "Option::is_none")]
    pub orig_qty: Option<Box<models::GetOrderModifyHistoryResponseInnerAmendmentOrigQty>>,
    #[serde(rename = "count", skip_serializing_if = "Option::is_none")]
    pub count: Option<i64>,
}

impl GetOrderModifyHistoryResponseInnerAmendment {
    #[must_use]
    pub fn new() -> GetOrderModifyHistoryResponseInnerAmendment {
        GetOrderModifyHistoryResponseInnerAmendment {
            price: None,
            orig_qty: None,
            count: None,
        }
    }
}
