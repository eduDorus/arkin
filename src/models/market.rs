use core::fmt;

use time::OffsetDateTime;

use super::{Instrument, Price, Quantity};

#[derive(Clone)]
pub enum MarketEvent {
    Tick(Tick),
    Trade(Trade),
    AggTrade(Trade),
    BookUpdate(BookUpdate),
}

impl fmt::Display for MarketEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MarketEvent::Tick(tick) => write!(f, "Tick: {}", tick),
            MarketEvent::Trade(trade) => write!(f, "Trade: {}", trade),
            MarketEvent::AggTrade(trade) => write!(f, "AggTrade: {}", trade),
            MarketEvent::BookUpdate(book_update) => write!(f, "Book Update: {}", book_update),
        }
    }
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

impl Tick {
    pub fn new(
        instrument: Instrument,
        event_time: OffsetDateTime,
        bid_price: Price,
        bid_quantity: Quantity,
        ask_price: Price,
        ask_quantity: Quantity,
    ) -> Self {
        Self {
            instrument,
            event_time,
            bid_price,
            bid_quantity,
            ask_price,
            ask_quantity,
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
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub price: Price,
    pub quantity: Quantity, // Negative for sell, positive for buy
}

impl Trade {
    pub fn new(instrument: Instrument, event_time: OffsetDateTime, price: Price, quantity: Quantity) -> Self {
        Self {
            instrument,
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
pub struct BookUpdate {
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub bids: Vec<BookUpdateSide>,
    pub asks: Vec<BookUpdateSide>,
}

impl fmt::Display for BookUpdate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.instrument, self.event_time)
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
