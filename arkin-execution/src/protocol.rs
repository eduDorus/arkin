use std::sync::Arc;

use arkin_core::prelude::*;
use strum::Display;

use crate::errors::ExecutionError;

pub enum ExecutorRequest {
    PlaceOrder(Order),
    CancelOrder(u64),
    CancelAllOrders,
    Shutdown,
}

pub enum ExecutorResponse {
    OrderUpdate(u64, OrderStatus),
    Error(ExecutionError),
}

#[derive(Debug, Clone, Display)]
pub enum OrderStatus {
    New,
    Send,
    Accepted,
    Rejected,
    Canceled,
    PartiallyFilled,
    Filled,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub instrument: Arc<Instrument>,
    pub side: MarketSide,
    pub price: f64,
    pub quantity: f64,
    pub status: OrderStatus,
}

impl Order {
    pub fn new(id: u64, instrument: Arc<Instrument>, side: MarketSide, price: f64, quantity: f64) -> Self {
        Self {
            id,
            instrument,
            side,
            price,
            quantity,
            status: OrderStatus::New,
        }
    }
}
