#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct BasisResponseInner {
    #[serde(rename = "indexPrice", skip_serializing_if = "Option::is_none")]
    pub index_price: Option<String>,
    #[serde(rename = "contractType", skip_serializing_if = "Option::is_none")]
    pub contract_type: Option<String>,
    #[serde(rename = "basisRate", skip_serializing_if = "Option::is_none")]
    pub basis_rate: Option<String>,
    #[serde(rename = "futuresPrice", skip_serializing_if = "Option::is_none")]
    pub futures_price: Option<String>,
    #[serde(
        rename = "annualizedBasisRate",
        skip_serializing_if = "Option::is_none"
    )]
    pub annualized_basis_rate: Option<String>,
    #[serde(rename = "basis", skip_serializing_if = "Option::is_none")]
    pub basis: Option<String>,
    #[serde(rename = "pair", skip_serializing_if = "Option::is_none")]
    pub pair: Option<String>,
    #[serde(rename = "timestamp", skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

impl BasisResponseInner {
    #[must_use]
    pub fn new() -> BasisResponseInner {
        BasisResponseInner {
            index_price: None,
            contract_type: None,
            basis_rate: None,
            futures_price: None,
            annualized_basis_rate: None,
            basis: None,
            pair: None,
            timestamp: None,
        }
    }
}
