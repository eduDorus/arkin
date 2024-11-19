use std::sync::Arc;

use strum::{Display, EnumDiscriminants, EnumString};
use time::OffsetDateTime;

use crate::{
    models::{Account, Book, ExecutionOrder, Fill, Position, Signal, Strategy, Tick, Trade, Venue, VenueOrder},
    Instrument,
};

pub trait EventTypeOf {
    fn event_type() -> EventType;
}

#[derive(Display, Clone, EnumDiscriminants)]
#[strum_discriminants(name(EventType))]
#[strum_discriminants(derive(EnumString, Display, Hash))]
pub enum Event {
    Venue(Venue),
    Account(Account),
    Strategy(Strategy),
    Tick(Tick),
    Trade(Trade),
    Book(Book),
    Signal(Signal),
    ExecutionOrder(ExecutionOrder),
    VenueOrder(VenueOrder),
    Fill(Fill),
    Position(Position),
}

impl Event {
    pub fn event_type(&self) -> EventType {
        self.into()
    }

    pub fn instrument(&self) -> Arc<Instrument> {
        match self {
            Event::Tick(tick) => tick.instrument.clone(),
            Event::Trade(trade) => trade.instrument.clone(),
            Event::Book(book) => book.instrument.clone(),
            Event::Signal(signal) => signal.instrument.clone(),
            Event::ExecutionOrder(order) => order.instrument.clone(),
            Event::VenueOrder(order) => order.instrument.clone(),
            Event::Fill(fill) => fill.instrument.clone(),
            Event::Position(position) => position.instrument.clone(),
            _ => panic!("Event does not have an instrument"),
        }
    }

    pub fn timestamp(&self) -> OffsetDateTime {
        match self {
            Event::Tick(tick) => tick.event_time,
            Event::Trade(trade) => trade.event_time,
            Event::Book(book) => book.event_time,
            Event::Signal(signal) => signal.event_time,
            Event::ExecutionOrder(_order) => OffsetDateTime::now_utc(),
            Event::VenueOrder(_order) => OffsetDateTime::now_utc(),
            Event::Fill(_fill) => OffsetDateTime::now_utc(),
            Event::Position(_position) => OffsetDateTime::now_utc(),
            _ => panic!("Event does not have a timestamp"),
        }
    }
}
