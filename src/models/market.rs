use core::fmt;

use time::OffsetDateTime;

use super::{Instrument, Price, Quantity};

#[derive(Clone)]
pub enum MarketEvent {
    Tick(Tick),
    Trade(Trade),
    AggTrade(Trade),
    // OrderBookUpdate(OrderBookUpdate),
}

impl fmt::Display for MarketEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MarketEvent::Tick(tick) => write!(f, "Tick: {}", tick),
            MarketEvent::Trade(trade) => write!(f, "Trade: {}", trade),
            MarketEvent::AggTrade(trade) => write!(f, "AggTrade: {}", trade),
            // MarketEvent::OrderBookUpdate(order_book_update) => write!(f, "{}", order_book_update),
        }
    }
}

#[derive(Clone)]
pub struct Tick {
    pub instrument: Instrument,
    pub transaction_time: OffsetDateTime,
    pub event_time: OffsetDateTime,
    pub bid_price: Price,
    pub bid_quantity: Quantity,
    pub ask_price: Price,
    pub ask_quantity: Quantity,
}

impl fmt::Display for Tick {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} bid: {} {} ask: {} {}",
            self.instrument, self.event_time, self.bid_price, self.bid_quantity, self.ask_price, self.ask_quantity
        )
    }
}

#[derive(Clone)]
pub struct Trade {
    pub instrument: Instrument,
    pub transaction_time: OffsetDateTime,
    pub event_time: OffsetDateTime,
    pub price: Price,
    pub quantity: Quantity, // Negative for sell, positive for buy
}

impl Trade {
    pub fn new(
        instrument: Instrument,
        transaction_time: OffsetDateTime,
        event_time: OffsetDateTime,
        price: Price,
        quantity: Quantity,
    ) -> Self {
        Self {
            instrument,
            transaction_time,
            event_time,
            price,
            quantity,
        }
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {}", self.instrument, self.event_time, self.price, self.quantity)
    }
}

#[derive(Clone)]
pub struct OrderBookUpdate {
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub side: OrderBookSide,
    pub price: Price,
    pub quantity: Quantity,
}

impl fmt::Display for OrderBookUpdate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.instrument, self.event_time, self.side, self.price, self.quantity
        )
    }
}

#[derive(Clone)]
pub enum OrderBookSide {
    Bid,
    Ask,
}

impl fmt::Display for OrderBookSide {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderBookSide::Bid => write!(f, "Bid"),
            OrderBookSide::Ask => write!(f, "Ask"),
        }
    }
}
