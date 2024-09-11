use strum::{Display, EnumDiscriminants, EnumString};

use crate::models::{Account, Book, ExecutionOrder, Fill, Position, Signal, Strategy, Tick, Trade, Venue, VenueOrder};

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
}
