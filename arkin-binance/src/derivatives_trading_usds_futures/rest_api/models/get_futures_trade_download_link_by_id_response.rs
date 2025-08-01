#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::rest_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct GetFuturesTradeDownloadLinkByIdResponse {
    #[serde(rename = "downloadId", skip_serializing_if = "Option::is_none")]
    pub download_id: Option<String>,
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(rename = "url", skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(rename = "notified", skip_serializing_if = "Option::is_none")]
    pub notified: Option<bool>,
    #[serde(
        rename = "expirationTimestamp",
        skip_serializing_if = "Option::is_none"
    )]
    pub expiration_timestamp: Option<i64>,
    #[serde(rename = "isExpired", skip_serializing_if = "Option::is_none")]
    pub is_expired: Option<String>,
}

impl GetFuturesTradeDownloadLinkByIdResponse {
    #[must_use]
    pub fn new() -> GetFuturesTradeDownloadLinkByIdResponse {
        GetFuturesTradeDownloadLinkByIdResponse {
            download_id: None,
            status: None,
            url: None,
            notified: None,
            expiration_timestamp: None,
            is_expired: None,
        }
    }
}
