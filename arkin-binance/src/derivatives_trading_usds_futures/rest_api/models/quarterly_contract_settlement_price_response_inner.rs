
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct QuarterlyContractSettlementPriceResponseInner {
    #[serde(rename = "deliveryTime", skip_serializing_if = "Option::is_none")]
    pub delivery_time: Option<i64>,
    #[serde(rename = "deliveryPrice", skip_serializing_if = "Option::is_none")]
    pub delivery_price: Option<rust_decimal::Decimal>,
}

impl QuarterlyContractSettlementPriceResponseInner {
    #[must_use]
    pub fn new() -> QuarterlyContractSettlementPriceResponseInner {
        QuarterlyContractSettlementPriceResponseInner {
            delivery_time: None,
            delivery_price: None,
        }
    }
}
