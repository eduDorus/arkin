#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IndexPriceKlineCandlestickDataResponseItemInner {
    Integer(i64),
    String(String),
    Other(serde_json::Value),
}

impl Default for IndexPriceKlineCandlestickDataResponseItemInner {
    fn default() -> Self {
        Self::Integer(Default::default())
    }
}
