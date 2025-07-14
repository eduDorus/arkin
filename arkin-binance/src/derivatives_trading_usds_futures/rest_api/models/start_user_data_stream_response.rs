
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct StartUserDataStreamResponse {
    #[serde(rename = "listenKey", skip_serializing_if = "Option::is_none")]
    pub listen_key: Option<String>,
}

impl StartUserDataStreamResponse {
    #[must_use]
    pub fn new() -> StartUserDataStreamResponse {
        StartUserDataStreamResponse { listen_key: None }
    }
}
