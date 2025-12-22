#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetCurrentPositionModeResponse {
    #[serde(rename = "dualSidePosition", skip_serializing_if = "Option::is_none")]
    pub dual_side_position: Option<bool>,
}

impl GetCurrentPositionModeResponse {
    #[must_use]
    pub fn new() -> GetCurrentPositionModeResponse {
        GetCurrentPositionModeResponse {
            dual_side_position: None,
        }
    }
}
