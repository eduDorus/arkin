
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryInsuranceFundBalanceSnapshotResponse1AssetsInner {
    #[serde(rename = "asset", skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    #[serde(rename = "marginBalance", skip_serializing_if = "Option::is_none")]
    pub margin_balance: Option<String>,
    #[serde(rename = "updateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<i64>,
}

impl QueryInsuranceFundBalanceSnapshotResponse1AssetsInner {
    #[must_use]
    pub fn new() -> QueryInsuranceFundBalanceSnapshotResponse1AssetsInner {
        QueryInsuranceFundBalanceSnapshotResponse1AssetsInner {
            asset: None,
            margin_balance: None,
            update_time: None,
        }
    }
}
