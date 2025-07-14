
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetOrderModifyHistoryResponseInnerAmendmentOrigQty {
    #[serde(rename = "before", skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(rename = "after", skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}

impl GetOrderModifyHistoryResponseInnerAmendmentOrigQty {
    #[must_use]
    pub fn new() -> GetOrderModifyHistoryResponseInnerAmendmentOrigQty {
        GetOrderModifyHistoryResponseInnerAmendmentOrigQty {
            before: None,
            after: None,
        }
    }
}
