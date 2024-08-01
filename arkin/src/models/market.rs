use crate::ingestors::IngestorID;

use super::{Instrument, Price, Quantity};
use std::fmt;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct Tick {
    pub received_time: OffsetDateTime,
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub tick_id: u64,
    pub bid_price: Price,
    pub bid_quantity: Quantity,
    pub ask_price: Price,
    pub ask_quantity: Quantity,
    pub source: IngestorID,
}

impl Tick {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Instrument,
        tick_id: u64,
        bid_price: Price,
        bid_quantity: Quantity,
        ask_price: Price,
        ask_quantity: Quantity,
        source: IngestorID,
    ) -> Self {
        Self {
            received_time: OffsetDateTime::now_utc(),
            event_time,
            instrument,
            tick_id,
            bid_price,
            bid_quantity,
            ask_price,
            ask_quantity,
            source,
        }
    }
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
    pub received_time: OffsetDateTime,
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub trade_id: u64,
    pub price: Price,
    pub quantity: Quantity, // Negative for sell, positive for buy
    pub source: IngestorID,
}

impl Trade {
    pub fn new(
        received_time: OffsetDateTime,
        event_time: OffsetDateTime,
        instrument: Instrument,
        trade_id: u64,
        price: Price,
        quantity: Quantity,
        source: IngestorID,
    ) -> Self {
        Self {
            received_time,
            event_time,
            instrument,
            trade_id,
            price,
            quantity,
            source,
        }
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} {} {}", self.instrument, self.event_time, self.price, self.quantity)
    }
}

#[derive(Clone)]
pub struct BookUpdate {
    pub received_time: OffsetDateTime,
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub bids: Vec<BookUpdateSide>,
    pub asks: Vec<BookUpdateSide>,
    pub source: IngestorID,
}

impl BookUpdate {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Instrument,
        bids: Vec<BookUpdateSide>,
        asks: Vec<BookUpdateSide>,
        source: IngestorID,
    ) -> Self {
        Self {
            received_time: OffsetDateTime::now_utc(),
            event_time,
            instrument,
            bids,
            asks,
            source,
        }
    }
}

impl fmt::Display for BookUpdate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} bid: {} ask: {}",
            self.instrument,
            self.event_time,
            self.bids.len(),
            self.asks.len()
        )
    }
}

#[derive(Clone)]
pub struct BookUpdateSide {
    pub price: Price,
    pub quantity: Quantity,
}

impl BookUpdateSide {
    pub fn new(price: Price, quantity: Quantity) -> Self {
        Self { price, quantity }
    }
}

impl fmt::Display for BookUpdateSide {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.price, self.quantity)
    }
}
