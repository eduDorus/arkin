#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CheckServerTimeResponse {
    #[serde(rename = "serverTime", skip_serializing_if = "Option::is_none")]
    pub server_time: Option<i64>,
}

impl CheckServerTimeResponse {
    #[must_use]
    pub fn new() -> CheckServerTimeResponse {
        CheckServerTimeResponse { server_time: None }
    }
}
