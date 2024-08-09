use crate::{constants::TIMESTAMP_FORMAT, strategies::StrategyId};

use super::{Event, EventType, EventTypeOf, Instrument, Notional, Price, Quantity, Venue};
use std::fmt;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct Account {
    pub name: String,
    pub venue: Venue,
}

#[derive(Clone)]
pub struct Position {
    pub strategy_id: StrategyId,
    pub instrument: Instrument,
    pub start_time: OffsetDateTime,
    pub exit_time: Option<OffsetDateTime>,
    pub entry_price: Price,
    pub exit_price: Option<Price>,
    pub avg_price: Price,
    pub quantity: Quantity,
    pub commission: Notional,
}

impl Position {
    pub fn new(
        strategy_id: StrategyId,
        instrument: Instrument,
        start_time: OffsetDateTime,
        entry_price: Price,
        quantity: Quantity,
    ) -> Self {
        Self {
            strategy_id,
            instrument,
            start_time,
            exit_time: None,
            entry_price,
            exit_price: None,
            avg_price: entry_price,
            quantity,
            commission: Notional::from(0.),
        }
    }

    pub fn from_fill(fill: &Fill) -> Self {
        Self {
            strategy_id: fill.strategy_id.clone(),
            instrument: fill.instrument.clone(),
            start_time: fill.event_time,
            exit_time: None,
            entry_price: fill.price,
            exit_price: None,
            avg_price: fill.price,
            quantity: fill.quantity,
            commission: fill.commission,
        }
    }
    pub fn update(&mut self, fill: &Fill) -> Option<Fill> {
        let new_quantity = self.quantity + fill.quantity;
        self.commission += fill.commission;

        match (new_quantity.is_zero(), self.quantity.is_positive(), new_quantity.is_negative()) {
            // Quantity is zero so we close the position
            (true, _, _) => {
                self.avg_price = (self.notional() + fill.notional()) / (self.quantity + fill.quantity.abs());
                self.exit_price = Some(fill.price);
                self.exit_time = Some(fill.event_time);
                None
            }
            // Position flips
            (_, true, false) | (_, false, true) => {
                let fillable = self.quantity - (fill.quantity - self.quantity);
                self.avg_price = (self.notional() + fill.price * fillable) / (self.quantity + fillable.abs());
                self.exit_price = Some(fill.price);
                self.exit_time = Some(fill.event_time);
                Some(Fill::new(
                    fill.event_time,
                    fill.instrument.clone(),
                    fill.order_id,
                    fill.strategy_id.clone(),
                    fill.price,
                    fill.quantity - fillable,
                    fill.commission * (fillable / fill.quantity),
                ))
            }
            // Position is still open
            _ => {
                self.avg_price = (self.notional() + fill.notional()) / (self.quantity + fill.quantity.abs());
                self.quantity = self.quantity + fill.quantity;
                None
            }
        }
    }

    pub fn notional(&self) -> Notional {
        self.avg_price * self.quantity
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "POSITION {} {} avg price: {} quantity: {}",
            self.start_time.format(TIMESTAMP_FORMAT).unwrap(),
            self.instrument,
            self.avg_price,
            self.quantity
        )
    }
}

#[derive(Clone)]
pub enum PositionStatus {
    Open,
    Closed,
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

impl EventTypeOf for Order {
    fn event_type() -> EventType {
        EventType::Order
    }
}

impl TryFrom<Event> for Order {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Order(order) = event {
            Ok(order)
        } else {
            Err(())
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
    pub order_id: u64,
    pub strategy_id: StrategyId,
    pub price: Price,
    pub quantity: Quantity,
    pub commission: Notional,
}

impl Fill {
    pub fn new(
        event_time: OffsetDateTime,
        instrument: Instrument,
        order_id: u64,
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

    pub fn notional(&self) -> Notional {
        self.price * self.quantity
    }
}

impl EventTypeOf for Fill {
    fn event_type() -> EventType {
        EventType::Fill
    }
}

impl TryFrom<Event> for Fill {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Fill(fill) = event {
            Ok(fill)
        } else {
            Err(())
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
