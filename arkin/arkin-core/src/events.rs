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

    pub fn instrument(&self) -> Instrument {
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
            Event::Signal(signal) => signal.created_at,
            Event::ExecutionOrder(order) => order.created_at,
            Event::VenueOrder(order) => order.created_at,
            Event::Fill(fill) => fill.created_at,
            Event::Position(position) => position.created_at,
            _ => panic!("Event does not have a timestamp"),
        }
    }
}
