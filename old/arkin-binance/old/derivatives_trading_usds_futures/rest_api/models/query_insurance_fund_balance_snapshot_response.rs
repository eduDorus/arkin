#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueryInsuranceFundBalanceSnapshotResponse {
    QueryInsuranceFundBalanceSnapshotResponse1(Box<models::QueryInsuranceFundBalanceSnapshotResponse1>),
    QueryInsuranceFundBalanceSnapshotResponse2(Vec<models::QueryInsuranceFundBalanceSnapshotResponse2Inner>),
    Other(serde_json::Value),
}

impl Default for QueryInsuranceFundBalanceSnapshotResponse {
    fn default() -> Self {
        Self::QueryInsuranceFundBalanceSnapshotResponse1(Default::default())
    }
}
