
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MarkPriceKlineCandlestickDataResponseItemInner {
    Integer(i64),
    String(String),
    Other(serde_json::Value),
}

impl Default for MarkPriceKlineCandlestickDataResponseItemInner {
    fn default() -> Self {
        Self::Integer(Default::default())
    }
}
