use crate::{constants::TIMESTAMP_FORMAT, strategies::StrategyId};

use super::{Instrument, Notional, Price, Quantity, Venue};
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
    pub avg_price: Price,
    pub quantity: Quantity,
}

impl Position {
    pub fn new(instrument: Instrument, event_time: OffsetDateTime) -> Self {
        Self {
            instrument,
            event_time,
            avg_price: Price::from(0.),
            quantity: Quantity::from(0.),
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "POSITION {} {} avg price: {} quantity: {}",
            self.event_time.format(TIMESTAMP_FORMAT).unwrap(),
            self.instrument,
            self.avg_price,
            self.quantity
        )
    }
}

#[derive(Clone)]
pub struct Order {
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub order_id: u64,
    pub strategy_id: StrategyId,
    pub order_type: OrderType,
    pub price: Option<Price>,
    pub avg_fill_price: Option<Price>,
    pub quantity: Quantity,
    pub quantity_filled: Quantity,
    pub status: OrderStatus,
}

impl Order {
    pub fn new_market(
        event_time: OffsetDateTime,
        instrument: Instrument,
        strategy_id: StrategyId,
        quantity: Quantity,
    ) -> Self {
        Self {
            event_time,
            instrument,
            order_id: 0,
            strategy_id,
            order_type: OrderType::Market,
            price: None,
            avg_fill_price: None,
            quantity,
            quantity_filled: Quantity::from(0.),
            status: OrderStatus::New,
        }
    }
}

impl fmt::Display for Order {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.status {
            OrderStatus::PartiallyFilled | OrderStatus::Filled => {
                write!(
                    f,
                    "ORDER {} {} type: {} avg price: {} quantity: {}/{} status: {}",
                    self.event_time,
                    self.instrument,
                    self.order_type,
                    self.avg_fill_price.expect("No fill price"),
                    self.quantity_filled,
                    self.quantity,
                    self.status,
                )
            }
            _ => {
                write!(
                    f,
                    "ORDER {} {} type: {} price: {} quantity: {} status: {}",
                    self.event_time.format(TIMESTAMP_FORMAT).unwrap(),
                    self.instrument,
                    self.order_type,
                    self.price.unwrap_or(Price::from(0.)),
                    self.quantity,
                    self.status
                )
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
    New,
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
            OrderStatus::New => write!(f, "new"),
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
    pub event_time: OffsetDateTime,
    pub instrument: Instrument,
    pub order_id: Option<u64>,
    pub strategy_id: StrategyId,
    pub price: Price,
    pub quantity: Quantity,
    pub commission: Notional,
}

impl Fill {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Instrument,
        order_id: Option<u64>,
        strategy_id: StrategyId,
        price: Price,
        quantity: Quantity,
        commission: Notional,
    ) -> Self {
        Self {
            event_time,
            instrument,
            order_id,
            strategy_id,
            price,
            quantity,
            commission,
        }
    }
}

impl fmt::Display for Fill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "FILL {} {} strategy: {} avg price: {} quantity: {} commission: {}",
            self.event_time.format(TIMESTAMP_FORMAT).unwrap(),
            self.instrument,
            self.strategy_id,
            self.price,
            self.quantity,
            self.commission
        )
    }
}
