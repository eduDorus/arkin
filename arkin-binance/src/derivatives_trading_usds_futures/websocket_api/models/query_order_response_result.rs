/*
 * Binance Derivatives Trading USDS Futures WebSocket API
 *
 * OpenAPI Specification for the Binance Derivatives Trading USDS Futures WebSocket API
 *
 * The version of the OpenAPI document: 1.0.0
 *
 *
 * NOTE: This class is auto generated by OpenAPI Generator (https://openapi-generator.tech).
 * https://openapi-generator.tech
 * Do not edit the class manually.
 */

#![allow(unused_imports)]
use crate::derivatives_trading_usds_futures::websocket_api::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryOrderResponseResult {
    #[serde(rename = "avgPrice", skip_serializing_if = "Option::is_none")]
    pub avg_price: Option<String>,
    #[serde(rename = "clientOrderId", skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,
    #[serde(rename = "cumQuote", skip_serializing_if = "Option::is_none")]
    pub cum_quote: Option<String>,
    #[serde(rename = "executedQty", skip_serializing_if = "Option::is_none")]
    pub executed_qty: Option<String>,
    #[serde(rename = "orderId", skip_serializing_if = "Option::is_none")]
    pub order_id: Option<i64>,
    #[serde(rename = "origQty", skip_serializing_if = "Option::is_none")]
    pub orig_qty: Option<String>,
    #[serde(rename = "origType", skip_serializing_if = "Option::is_none")]
    pub orig_type: Option<String>,
    #[serde(rename = "price", skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    #[serde(rename = "reduceOnly", skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    #[serde(rename = "side", skip_serializing_if = "Option::is_none")]
    pub side: Option<String>,
    #[serde(rename = "positionSide", skip_serializing_if = "Option::is_none")]
    pub position_side: Option<String>,
    #[serde(rename = "status", skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(rename = "stopPrice", skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<String>,
    #[serde(rename = "closePosition", skip_serializing_if = "Option::is_none")]
    pub close_position: Option<bool>,
    #[serde(rename = "symbol", skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(rename = "time", skip_serializing_if = "Option::is_none")]
    pub time: Option<i64>,
    #[serde(rename = "timeInForce", skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "activatePrice", skip_serializing_if = "Option::is_none")]
    pub activate_price: Option<String>,
    #[serde(rename = "priceRate", skip_serializing_if = "Option::is_none")]
    pub price_rate: Option<String>,
    #[serde(rename = "updateTime", skip_serializing_if = "Option::is_none")]
    pub update_time: Option<i64>,
    #[serde(rename = "workingType", skip_serializing_if = "Option::is_none")]
    pub working_type: Option<String>,
    #[serde(rename = "priceProtect", skip_serializing_if = "Option::is_none")]
    pub price_protect: Option<bool>,
}

impl QueryOrderResponseResult {
    #[must_use]
    pub fn new() -> QueryOrderResponseResult {
        QueryOrderResponseResult {
            avg_price: None,
            client_order_id: None,
            cum_quote: None,
            executed_qty: None,
            order_id: None,
            orig_qty: None,
            orig_type: None,
            price: None,
            reduce_only: None,
            side: None,
            position_side: None,
            status: None,
            stop_price: None,
            close_position: None,
            symbol: None,
            time: None,
            time_in_force: None,
            r#type: None,
            activate_price: None,
            price_rate: None,
            update_time: None,
            working_type: None,
            price_protect: None,
        }
    }
}
