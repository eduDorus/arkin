use strum::{Display, EnumDiscriminants, EnumString};
use time::OffsetDateTime;

use super::{Allocation, Book, ExecutionOrder, Fill, Instrument, Signal, Tick, Trade};

pub trait EventTypeOf {
    fn event_type() -> EventType;
}

#[derive(Display, Clone, EnumDiscriminants)]
#[strum_discriminants(name(EventType))]
#[strum_discriminants(derive(EnumString, Display, Hash))]
pub enum Event {
    Tick(Tick),
    Trade(Trade),
    Book(Book),
    Signal(Signal),
    Allocation(Allocation),
    ExecutionOrder(ExecutionOrder),
    Fill(Fill),
    CompletedTrade(Trade),
}

impl Event {
    pub fn event_time(&self) -> &OffsetDateTime {
        match self {
            Event::Tick(e) => &e.event_time,
            Event::Trade(e) => &e.event_time,
            Event::Book(e) => &e.event_time,
            Event::Signal(e) => &e.event_time,
            Event::Allocation(e) => &e.event_time,
            Event::ExecutionOrder(e) => &e.event_time,
            Event::Fill(e) => &e.event_time,
            Event::CompletedTrade(e) => &e.event_time,
        }
    }

    pub fn instrument(&self) -> &Instrument {
        match self {
            Event::Tick(e) => &e.instrument,
            Event::Trade(e) => &e.instrument,
            Event::Book(e) => &e.instrument,
            Event::Signal(e) => &e.instrument,
            Event::Allocation(e) => &e.instrument,
            Event::ExecutionOrder(e) => &e.instrument,
            Event::Fill(e) => &e.instrument,
            Event::CompletedTrade(e) => &e.instrument,
        }
    }

    pub fn event_type(&self) -> EventType {
        self.into()
    }
}
