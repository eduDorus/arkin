#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompositeIndexSymbolInformationResponseInnerBaseAssetListInner {
    #[serde(rename = "baseAsset", skip_serializing_if = "Option::is_none")]
    pub base_asset: Option<String>,
    #[serde(rename = "quoteAsset", skip_serializing_if = "Option::is_none")]
    pub quote_asset: Option<String>,
    #[serde(rename = "weightInQuantity", skip_serializing_if = "Option::is_none")]
    pub weight_in_quantity: Option<String>,
    #[serde(rename = "weightInPercentage", skip_serializing_if = "Option::is_none")]
    pub weight_in_percentage: Option<String>,
}

impl CompositeIndexSymbolInformationResponseInnerBaseAssetListInner {
    #[must_use]
    pub fn new() -> CompositeIndexSymbolInformationResponseInnerBaseAssetListInner {
        CompositeIndexSymbolInformationResponseInnerBaseAssetListInner {
            base_asset: None,
            quote_asset: None,
            weight_in_quantity: None,
            weight_in_percentage: None,
        }
    }
}
