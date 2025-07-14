
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MultiAssetsModeAssetIndexResponse {
    MultiAssetsModeAssetIndexResponse1(Box<models::MultiAssetsModeAssetIndexResponse1>),
    MultiAssetsModeAssetIndexResponse2(Vec<models::MultiAssetsModeAssetIndexResponse2Inner>),
    Other(serde_json::Value),
}

impl Default for MultiAssetsModeAssetIndexResponse {
    fn default() -> Self {
        Self::MultiAssetsModeAssetIndexResponse1(Default::default())
    }
}
