
#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetDownloadIdForFuturesTradeHistoryResponse {
    #[serde(
        rename = "avgCostTimestampOfLast30d",
        skip_serializing_if = "Option::is_none"
    )]
    pub avg_cost_timestamp_of_last30d: Option<i64>,
    #[serde(rename = "downloadId", skip_serializing_if = "Option::is_none")]
    pub download_id: Option<String>,
}

impl GetDownloadIdForFuturesTradeHistoryResponse {
    #[must_use]
    pub fn new() -> GetDownloadIdForFuturesTradeHistoryResponse {
        GetDownloadIdForFuturesTradeHistoryResponse {
            avg_cost_timestamp_of_last30d: None,
            download_id: None,
        }
    }
}
