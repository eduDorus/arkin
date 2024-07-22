use std::fmt;

use strum::EnumIter;
use time::OffsetDateTime;

use crate::features::VWAP;

use super::{BookUpdate, Fill, Instrument, Order, Position, Tick, Trade};

#[derive(Clone)]
pub enum Event {
    // Market
    TickUpdate(Tick),
    TradeUpdate(Trade),
    BookUpdate(BookUpdate),

    // Account
    PositionUpdate(Position),
    OrderUpdate(Order),
    FillUpdate(Fill),

    // Features
    VWAP(VWAP),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash, EnumIter)]
pub enum EventID {
    // Market
    TickUpdate,
    TradeUpdate,
    AggTradeUpdate,
    BookUpdate,

    // Account
    PositionUpdate,
    OrderUpdate,
    FillUpdate,

    // Features
    VWAP,
}

impl fmt::Display for EventID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            // Market
            EventID::TickUpdate => write!(f, "Tick Update"),
            EventID::TradeUpdate => write!(f, "Trade Update"),
            EventID::AggTradeUpdate => write!(f, "Agg trade update"),
            EventID::BookUpdate => write!(f, "Book update"),

            // Account
            EventID::PositionUpdate => write!(f, "Position update"),
            EventID::OrderUpdate => write!(f, "Order update"),
            EventID::FillUpdate => write!(f, "Fill update"),

            // Features
            EventID::VWAP => write!(f, "VWAP update"),
        }
    }
}

impl Event {
    // Function to match the type on
    pub fn event_time(&self) -> &OffsetDateTime {
        match self {
            // Market
            Event::TickUpdate(e) => &e.event_time,
            Event::TradeUpdate(e) => &e.event_time,
            Event::BookUpdate(e) => &e.event_time,

            // Account
            Event::PositionUpdate(e) => &e.event_time,
            Event::OrderUpdate(e) => &e.event_time,
            Event::FillUpdate(e) => &e.event_time,

            // Features
            Event::VWAP(e) => &e.event_time,
        }
    }

    pub fn instrument(&self) -> &Instrument {
        match self {
            // Market
            Event::TickUpdate(e) => &e.instrument,
            Event::TradeUpdate(e) => &e.instrument,
            Event::BookUpdate(e) => &e.instrument,

            // Account
            Event::PositionUpdate(e) => &e.instrument,
            Event::OrderUpdate(e) => &e.instrument,
            Event::FillUpdate(e) => &e.instrument,

            // Features
            Event::VWAP(e) => &e.instrument,
        }
    }

    pub fn event_type(&self) -> &EventID {
        match self {
            // Market
            Event::TickUpdate(_) => &EventID::TickUpdate,
            Event::TradeUpdate(_) => &EventID::TradeUpdate,
            Event::BookUpdate(_) => &EventID::BookUpdate,

            // Account
            Event::PositionUpdate(_) => &EventID::PositionUpdate,
            Event::OrderUpdate(_) => &EventID::OrderUpdate,
            Event::FillUpdate(_) => &EventID::FillUpdate,

            // Features
            Event::VWAP(_) => &EventID::VWAP,
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Event::TickUpdate(tick) => write!(f, "Tick update: {}", tick),
            Event::TradeUpdate(trade) => write!(f, "Trade update: {}", trade),
            Event::BookUpdate(book) => write!(f, "Book update: {}", book),
            Event::PositionUpdate(position) => write!(f, "Position update: {}", position),
            Event::OrderUpdate(order) => write!(f, "Order update: {}", order),
            Event::FillUpdate(fill) => write!(f, "Fill update: {}", fill),
            Event::VWAP(vwap) => write!(f, "VWAP update: {}", vwap),
        }
    }
}
