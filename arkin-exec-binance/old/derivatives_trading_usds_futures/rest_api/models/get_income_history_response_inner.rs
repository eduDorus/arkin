#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetIncomeHistoryResponseInner {
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "incomeType", skip_serializing_if = "Option::is_none")]
    pub income_type: Option<String>,
    #[serde(rename = "income", skip_serializing_if = "Option::is_none")]
    pub income: Option<String>,
    #[serde(rename = "asset", skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    #[serde(rename = "info", skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    #[serde(rename = "tranId", skip_serializing_if = "Option::is_none")]
    pub tran_id: Option<i64>,
    #[serde(rename = "tradeId", skip_serializing_if = "Option::is_none")]
    pub trade_id: Option<String>,
}

impl GetIncomeHistoryResponseInner {
    #[must_use]
    pub fn new() -> GetIncomeHistoryResponseInner {
        GetIncomeHistoryResponseInner {
            symbol: None,
            income_type: None,
            income: None,
            asset: None,
            info: None,
            time: None,
            tran_id: None,
            trade_id: None,
        }
    }
}
