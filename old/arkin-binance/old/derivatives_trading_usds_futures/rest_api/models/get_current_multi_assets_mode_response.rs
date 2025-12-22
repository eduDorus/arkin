#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetCurrentMultiAssetsModeResponse {
    #[serde(rename = "multiAssetsMargin", skip_serializing_if = "Option::is_none")]
    pub multi_assets_margin: Option<bool>,
}

impl GetCurrentMultiAssetsModeResponse {
    #[must_use]
    pub fn new() -> GetCurrentMultiAssetsModeResponse {
        GetCurrentMultiAssetsModeResponse {
            multi_assets_margin: None,
        }
    }
}
