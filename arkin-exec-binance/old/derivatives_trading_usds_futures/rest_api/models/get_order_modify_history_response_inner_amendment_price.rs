#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetOrderModifyHistoryResponseInnerAmendmentPrice {
    #[serde(rename = "before", skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(rename = "after", skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}

impl GetOrderModifyHistoryResponseInnerAmendmentPrice {
    #[must_use]
    pub fn new() -> GetOrderModifyHistoryResponseInnerAmendmentPrice {
        GetOrderModifyHistoryResponseInnerAmendmentPrice {
            before: None,
            after: None,
        }
    }
}
