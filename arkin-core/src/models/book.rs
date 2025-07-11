use std::{fmt, sync::Arc};

use time::UtcDateTime;

use crate::types::{Price, Quantity};

use super::Instrument;

#[derive(Debug, Clone)]
pub struct Book {
    pub received_time: UtcDateTime,
    pub event_time: UtcDateTime,
    pub instrument: Arc<Instrument>,
    pub bids: Vec<BookUpdateSide>,
    pub asks: Vec<BookUpdateSide>,
}

impl Book {
    pub fn new(
        event_time: UtcDateTime,
        instrument: Arc<Instrument>,
        bids: Vec<BookUpdateSide>,
        asks: Vec<BookUpdateSide>,
    ) -> Self {
        Self {
            received_time: UtcDateTime::now(),
            event_time,
            instrument,
            bids,
            asks,
        }
    }
}

impl fmt::Display for Book {
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

#[derive(Debug, Clone)]
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
