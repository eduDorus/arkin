use strum::{Display, EnumDiscriminants, EnumString};
use time::OffsetDateTime;

use super::{Allocation, Book, Fill, Instrument, Order, Signal, Tick, Trade};

pub trait EventTypeOf {
    fn event_type() -> EventType;
}

#[derive(Display, Clone, EnumDiscriminants)]
#[strum_discriminants(name(EventType))]
#[strum_discriminants(derive(Hash, EnumString, Display))]
pub enum Event {
    Tick(Tick),
    Trade(Trade),
    Book(Book),
    Order(Order),
    Fill(Fill),
    Signal(Signal),
    Allocation(Allocation),
}

impl Event {
    // Function to match the type on
    pub fn event_time(&self) -> &OffsetDateTime {
        match self {
            Event::Tick(e) => &e.event_time,
            Event::Trade(e) => &e.event_time,
            Event::Book(e) => &e.event_time,
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
            Event::Order(e) => &e.instrument,
            Event::Fill(e) => &e.instrument,
            Event::Signal(e) => &e.instrument,
            Event::Allocation(e) => &e.instrument,
        }
    }

    pub fn event_type(&self) -> EventType {
        self.into()
    }
}
