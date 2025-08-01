/*
 * Binance Derivatives Trading USDS Futures WebSocket Market Streams
 *
 * OpenAPI Specification for the Binance Derivatives Trading USDS Futures WebSocket Market Streams
 *
 * The version of the OpenAPI document: 1.0.0
 *
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::websocket_streams::models;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use serde_json::Value;

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContinuousContractKlineCandlestickStreamsResponse {
    #[serde(rename = "e", skip_serializing_if = "Option::is_none")]
    pub e: Option<String>,
    #[serde(rename = "E", skip_serializing_if = "Option::is_none")]
    pub e_uppercase: Option<i64>,
    #[serde(rename = "ps", skip_serializing_if = "Option::is_none")]
    pub ps: Option<String>,
    #[serde(rename = "ct", skip_serializing_if = "Option::is_none")]
    pub ct: Option<String>,
    #[serde(rename = "k", skip_serializing_if = "Option::is_none")]
    pub k: Option<Box<models::ContinuousContractKlineCandlestickStreamsResponseK>>,
}

impl ContinuousContractKlineCandlestickStreamsResponse {
    #[must_use]
    pub fn new() -> ContinuousContractKlineCandlestickStreamsResponse {
        ContinuousContractKlineCandlestickStreamsResponse {
            e: None,
            e_uppercase: None,
            ps: None,
            ct: None,
            k: None,
        }
    }
}
