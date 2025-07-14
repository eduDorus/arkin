
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExchangeInformationResponseAssetsInner {
    #[serde(rename = "asset", skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    #[serde(rename = "marginAvailable", skip_serializing_if = "Option::is_none")]
    pub margin_available: Option<bool>,
    #[serde(rename = "autoAssetExchange", skip_serializing_if = "Option::is_none")]
    pub auto_asset_exchange: Option<String>,
}

impl ExchangeInformationResponseAssetsInner {
    #[must_use]
    pub fn new() -> ExchangeInformationResponseAssetsInner {
        ExchangeInformationResponseAssetsInner {
            asset: None,
            margin_available: None,
            auto_asset_exchange: None,
        }
    }
}
