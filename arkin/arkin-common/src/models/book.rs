use std::fmt;

use time::OffsetDateTime;

use crate::{
    events::{Event, EventType, EventTypeOf},
    types::{Price, Quantity},
};

use super::Instrument;

#[derive(Clone)]
pub struct Book {
    pub received_time: OffsetDateTime,
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub bids: Vec<BookUpdateSide>,
    pub asks: Vec<BookUpdateSide>,
}

impl Book {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Instrument,
        bids: Vec<BookUpdateSide>,
        asks: Vec<BookUpdateSide>,
    ) -> Self {
        Self {
            received_time: OffsetDateTime::now_utc(),
            event_time,
            instrument,
            bids,
            asks,
        }
    }
}

impl EventTypeOf for Book {
    fn event_type() -> EventType {
        EventType::Book
    }
}

impl TryFrom<Event> for Book {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Book(book) = event {
            Ok(book)
        } else {
            Err(())
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