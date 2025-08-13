use arkin_core::prelude::{MarketSide, Price, Quantity};
use serde::{Deserialize, Serialize};

use crate::usdm::models::user_models::{BinanceOrderSide, BinanceOrderType, BinanceTimeInForce};

#[derive(Debug, Serialize, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewOrderRequest {
    pub symbol: String,
    pub side: BinanceOrderSide,
    pub order_type: BinanceOrderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<BinanceTimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Quantity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Price>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_position: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation_price: Option<Price>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_rate: Option<Price>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_protect: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_order_resp_type: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BinanceOrder {
    pub order_id: i64,
    pub symbol: String,
    pub status: String,
    pub client_order_id: String,
    pub price: Price,
    pub avg_price: Price,
    pub orig_qty: Quantity,
    pub executed_qty: Quantity,
    pub cum_quote: Quantity,
    pub time_in_force: String,
    pub r#type: String,
    pub reduce_only: bool,
    pub close_position: bool,
    pub side: MarketSide,
    pub position_side: String,
    pub stop_price: Price,
    pub working_type: String,
    pub price_protect: bool,
    pub orig_type: String,
    pub update_time: i64,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub order_id: i64,
    pub symbol: String,
    pub status: String,
    pub client_order_id: String,
    pub price: Price,
    pub avg_price: Price,
    pub orig_qty: Quantity,
    pub executed_qty: Quantity,
    pub cum_quote: Quantity,
    pub time_in_force: String,
    pub r#type: String,
    pub reduce_only: bool,
    pub close_position: bool,
    pub side: MarketSide,
    pub position_side: String,
    pub stop_price: Price,
    pub working_type: String,
    pub price_protect: bool,
    pub orig_type: String,
    pub update_time: i64,
}
