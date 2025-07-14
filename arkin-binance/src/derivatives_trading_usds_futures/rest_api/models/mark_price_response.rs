
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MarkPriceResponse {
    MarkPriceResponse1(Box<models::MarkPriceResponse1>),
    MarkPriceResponse2(Vec<models::MarkPriceResponse2Inner>),
    Other(serde_json::Value),
}

impl Default for MarkPriceResponse {
    fn default() -> Self {
        Self::MarkPriceResponse1(Default::default())
    }
}
