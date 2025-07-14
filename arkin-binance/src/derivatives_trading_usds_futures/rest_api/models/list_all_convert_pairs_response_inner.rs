
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ListAllConvertPairsResponseInner {
    #[serde(rename = "fromAsset", skip_serializing_if = "Option::is_none")]
    pub from_asset: Option<String>,
    #[serde(rename = "toAsset", skip_serializing_if = "Option::is_none")]
    pub to_asset: Option<String>,
    #[serde(rename = "fromAssetMinAmount", skip_serializing_if = "Option::is_none")]
    pub from_asset_min_amount: Option<String>,
    #[serde(rename = "fromAssetMaxAmount", skip_serializing_if = "Option::is_none")]
    pub from_asset_max_amount: Option<String>,
    #[serde(rename = "toAssetMinAmount", skip_serializing_if = "Option::is_none")]
    pub to_asset_min_amount: Option<String>,
    #[serde(rename = "toAssetMaxAmount", skip_serializing_if = "Option::is_none")]
    pub to_asset_max_amount: Option<String>,
}

impl ListAllConvertPairsResponseInner {
    #[must_use]
    pub fn new() -> ListAllConvertPairsResponseInner {
        ListAllConvertPairsResponseInner {
            from_asset: None,
            to_asset: None,
            from_asset_min_amount: None,
            from_asset_max_amount: None,
            to_asset_min_amount: None,
            to_asset_max_amount: None,
        }
    }
}
