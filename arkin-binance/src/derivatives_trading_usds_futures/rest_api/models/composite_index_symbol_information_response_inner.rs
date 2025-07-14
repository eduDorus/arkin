
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct CompositeIndexSymbolInformationResponseInner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    #[serde(rename = "component", skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    #[serde(rename = "baseAssetList", skip_serializing_if = "Option::is_none")]
    pub base_asset_list: Option<Vec<models::CompositeIndexSymbolInformationResponseInnerBaseAssetListInner>>,
}

impl CompositeIndexSymbolInformationResponseInner {
    #[must_use]
    pub fn new() -> CompositeIndexSymbolInformationResponseInner {
        CompositeIndexSymbolInformationResponseInner {
            symbol: None,
            time: None,
            component: None,
            base_asset_list: None,
        }
    }
}
