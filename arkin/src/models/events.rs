use std::fmt;
use time::OffsetDateTime;

use super::{AllocationEvent, BookUpdate, Fill, Instrument, Order, Position, Signal, Tick, Trade};

#[derive(Clone)]
pub enum Event {
    Tick(Tick),
    Trade(Trade),
    Book(BookUpdate),
    Position(Position),
    Order(Order),
    Fill(Fill),
    Signal(Signal),
    Allocation(AllocationEvent),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum EventType {
    Tick,
    Trade,
    Book,
    Position,
    Order,
    Fill,
    Signal,
    Allocation,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EventType::Tick => write!(f, "Tick"),
            EventType::Trade => write!(f, "Trade"),
            EventType::Book => write!(f, "Book"),
            EventType::Position => write!(f, "Position"),
            EventType::Order => write!(f, "Order"),
            EventType::Fill => write!(f, "Fill"),
            EventType::Signal => write!(f, "Signal"),
            EventType::Allocation => write!(f, "Allocation"),
        }
    }
}

impl Event {
    // Function to match the type on
    pub fn event_time(&self) -> &OffsetDateTime {
        match self {
            Event::Tick(e) => &e.event_time,
            Event::Trade(e) => &e.event_time,
            Event::Book(e) => &e.event_time,
            Event::Position(e) => &e.event_time,
            Event::Order(e) => &e.event_time,
            Event::Fill(e) => &e.event_time,
            Event::Signal(e) => &e.event_time,
            Event::Allocation(e) => &e.event_time,
        }
    }

    pub fn instrument(&self) -> &Instrument {
        match self {
            Event::Tick(e) => &e.instrument,
            Event::Trade(e) => &e.instrument,
            Event::Book(e) => &e.instrument,
            Event::Position(e) => &e.instrument,
            Event::Order(e) => &e.instrument,
            Event::Fill(e) => &e.instrument,
            Event::Signal(e) => &e.instrument,
            Event::Allocation(e) => &e.instrument,
        }
    }

    pub fn event_type(&self) -> &EventType {
        match self {
            Event::Tick(_) => &EventType::Tick,
            Event::Trade(_) => &EventType::Trade,
            Event::Book(_) => &EventType::Book,
            Event::Position(_) => &EventType::Position,
            Event::Order(_) => &EventType::Order,
            Event::Fill(_) => &EventType::Fill,
            Event::Signal(_) => &EventType::Signal,
            Event::Allocation(_) => &EventType::Allocation,
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Event::Tick(tick) => write!(f, "Tick: {}", tick),
            Event::Trade(trade) => write!(f, "Trade: {}", trade),
            Event::Book(book) => write!(f, "Book: {}", book),
            Event::Position(position) => write!(f, "Position: {}", position),
            Event::Order(order) => write!(f, "Order: {}", order),
            Event::Fill(fill) => write!(f, "Fill: {}", fill),
            Event::Signal(signal) => write!(f, "Signal: {}", signal),
            Event::Allocation(allocation) => write!(f, "Allocation: {}", allocation),
        }
    }
}
