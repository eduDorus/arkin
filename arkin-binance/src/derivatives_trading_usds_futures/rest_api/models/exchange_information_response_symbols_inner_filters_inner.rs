#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExchangeInformationResponseSymbolsInnerFiltersInner {
    #[serde(rename = "filterType", skip_serializing_if = "Option::is_none")]
    pub filter_type: Option<String>,
    #[serde(rename = "maxPrice", skip_serializing_if = "Option::is_none")]
    pub max_price: Option<String>,
    #[serde(rename = "minPrice", skip_serializing_if = "Option::is_none")]
    pub min_price: Option<String>,
    #[serde(rename = "tickSize", skip_serializing_if = "Option::is_none")]
    pub tick_size: Option<String>,
    #[serde(rename = "maxQty", skip_serializing_if = "Option::is_none")]
    pub max_qty: Option<String>,
    #[serde(rename = "minQty", skip_serializing_if = "Option::is_none")]
    pub min_qty: Option<String>,
    #[serde(rename = "stepSize", skip_serializing_if = "Option::is_none")]
    pub step_size: Option<String>,
    #[serde(rename = "limit", skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(rename = "notional", skip_serializing_if = "Option::is_none")]
    pub notional: Option<String>,
    #[serde(rename = "multiplierUp", skip_serializing_if = "Option::is_none")]
    pub multiplier_up: Option<String>,
    #[serde(rename = "multiplierDown", skip_serializing_if = "Option::is_none")]
    pub multiplier_down: Option<String>,
    #[serde(rename = "multiplierDecimal", skip_serializing_if = "Option::is_none")]
    pub multiplier_decimal: Option<String>,
}

impl ExchangeInformationResponseSymbolsInnerFiltersInner {
    #[must_use]
    pub fn new() -> ExchangeInformationResponseSymbolsInnerFiltersInner {
        ExchangeInformationResponseSymbolsInnerFiltersInner {
            filter_type: None,
            max_price: None,
            min_price: None,
            tick_size: None,
            max_qty: None,
            min_qty: None,
            step_size: None,
            limit: None,
            notional: None,
            multiplier_up: None,
            multiplier_down: None,
            multiplier_decimal: None,
        }
    }
}
