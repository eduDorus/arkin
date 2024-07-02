use time::OffsetDateTime;

use super::{Instrument, Price, Quantity};

#[derive(Clone)]
pub enum MarketEvent {
    Tick(Tick),
    Trade(Trade),
    AggTrade(Trade),
    // OrderBookUpdate(OrderBookUpdate),
}

#[derive(Clone)]
pub struct Tick {
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub bid_price: Price,
    pub bid_quantity: Quantity,
    pub ask_price: Price,
    pub ask_quantity: Quantity,
}

#[derive(Clone)]
pub struct Trade {
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub price: Price,
    pub quantity: Quantity, // Negative for sell, positive for buy
}

#[derive(Clone)]
pub struct OrderBookUpdate {
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub side: OrderBookSide,
    pub price: Price,
    pub quantity: Quantity,
}

#[derive(Clone)]
pub enum OrderBookSide {
    Bid,
    Ask,
}
