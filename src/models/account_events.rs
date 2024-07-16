use core::fmt;

use time::OffsetDateTime;

use super::{Instrument, Price, Quantity, Venue};

#[derive(Clone)]
pub struct Account {
    pub name: String,
    pub venue: Venue,
}

#[derive(Clone)]
pub enum AccountEvent {
    PositionUpdate(Position),
    OrderUpdate(Order),
    FillUpdate(Fill),
}

impl fmt::Display for AccountEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountEvent::PositionUpdate(position) => write!(f, "Position update: {}", position),
            AccountEvent::OrderUpdate(order) => write!(f, "Order update: {}", order),
            AccountEvent::FillUpdate(fill) => write!(f, "Fill update: {}", fill),
        }
    }
}

#[derive(Clone)]
pub struct Position {
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub quantity: Quantity,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.instrument, self.quantity)
    }
}

#[derive(Clone)]
pub struct Order {
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub state: OrderState,
    pub filled_quantity: Quantity,
    pub open_quantity: Quantity,
    pub average_fill_price: Price,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} filled {} of {} with avg price {}",
            self.instrument, self.state, self.filled_quantity, self.open_quantity, self.average_fill_price
        )
    }
}

#[derive(Clone)]
pub enum OrderState {
    Send,
    Open,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
}

impl fmt::Display for OrderState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderState::Send => write!(f, "SEND"),
            OrderState::Open => write!(f, "OPEN"),
            OrderState::PartiallyFilled => write!(f, "PARTIALLY FILLED"),
            OrderState::Filled => write!(f, "FILLED"),
            OrderState::Canceled => write!(f, "CANCELED"),
            OrderState::Rejected => write!(f, "REJECTED"),
        }
    }
}

#[derive(Clone)]
pub struct Fill {
    pub instrument: Instrument,
    pub event_time: OffsetDateTime,
    pub price: Price,
    pub quantity: Quantity,
}

impl fmt::Display for Fill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} at {}", self.instrument, self.quantity, self.price)
    }
}
