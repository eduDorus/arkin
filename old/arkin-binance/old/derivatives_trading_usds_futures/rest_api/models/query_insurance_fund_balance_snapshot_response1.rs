#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryInsuranceFundBalanceSnapshotResponse1 {
    #[serde(rename = "symbols", skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(rename = "assets", skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<models::QueryInsuranceFundBalanceSnapshotResponse1AssetsInner>>,
}

impl QueryInsuranceFundBalanceSnapshotResponse1 {
    #[must_use]
    pub fn new() -> QueryInsuranceFundBalanceSnapshotResponse1 {
        QueryInsuranceFundBalanceSnapshotResponse1 {
            symbols: None,
            assets: None,
        }
    }
}
