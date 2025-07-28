#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetBnbBurnStatusResponse {
    #[serde(rename = "feeBurn", skip_serializing_if = "Option::is_none")]
    pub fee_burn: Option<bool>,
}

impl GetBnbBurnStatusResponse {
    #[must_use]
    pub fn new() -> GetBnbBurnStatusResponse {
        GetBnbBurnStatusResponse { fee_burn: None }
    }
}
