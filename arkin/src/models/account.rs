use super::{Instrument, Price, Quantity, Venue};
use std::fmt;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct Account {
    pub name: String,
    pub venue: Venue,
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
    pub received_time: OffsetDateTime,
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub order_id: u64,
    pub strategy_id: String,
    pub order_type: OrderType,
    pub price: Option<Price>,
    pub avg_fill_price: Option<Price>,
    pub quantity: Quantity,
    pub quantity_filled: Quantity,
    pub status: OrderStatus,
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.status {
            OrderStatus::PartiallyFilled | OrderStatus::Filled => {
                write!(
                    f,
                    "{} {} filled {} of {} with avg price {}",
                    self.instrument,
                    self.status,
                    self.quantity_filled,
                    self.quantity,
                    self.avg_fill_price.expect("No fill price"),
                )
            }
            _ => {
                write!(f, "{} {} {} {}", self.instrument, self.order_id, self.order_type, self.status)
            }
        }
    }
}

#[derive(Clone)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
}

impl fmt::Display for OrderType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderType::Market => write!(f, "market"),
            OrderType::Limit => write!(f, "limit"),
            OrderType::Stop => write!(f, "stop"),
            OrderType::StopLimit => write!(f, "stop_limit"),
        }
    }
}

#[derive(Clone)]
pub enum OrderStatus {
    Send,
    Open,
    PartiallyFilled,
    Filled,
    Canceled,
    Rejected,
}

impl fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OrderStatus::Send => write!(f, "send"),
            OrderStatus::Open => write!(f, "open"),
            OrderStatus::PartiallyFilled => write!(f, "partially_filled"),
            OrderStatus::Filled => write!(f, "filled"),
            OrderStatus::Canceled => write!(f, "canceled"),
            OrderStatus::Rejected => write!(f, "rejected"),
        }
    }
}

#[derive(Clone)]
pub struct Fill {
    pub received_time: OffsetDateTime,
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub order_id: Option<u64>,
    pub strategy_id: String,
    pub price: Price,
    pub quantity: Quantity,
    pub commission: Price,
}

impl fmt::Display for Fill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {} at {}", self.instrument, self.quantity, self.price)
    }
}
