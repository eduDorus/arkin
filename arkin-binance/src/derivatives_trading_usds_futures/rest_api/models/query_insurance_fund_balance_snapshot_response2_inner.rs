#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryInsuranceFundBalanceSnapshotResponse2Inner {
    #[serde(rename = "symbols", skip_serializing_if = "Option::is_none")]
    pub symbols: Option<Vec<String>>,
    #[serde(rename = "assets", skip_serializing_if = "Option::is_none")]
    pub assets: Option<Vec<models::QueryInsuranceFundBalanceSnapshotResponse2InnerAssetsInner>>,
}

impl QueryInsuranceFundBalanceSnapshotResponse2Inner {
    #[must_use]
    pub fn new() -> QueryInsuranceFundBalanceSnapshotResponse2Inner {
        QueryInsuranceFundBalanceSnapshotResponse2Inner {
            symbols: None,
            assets: None,
        }
    }
}
