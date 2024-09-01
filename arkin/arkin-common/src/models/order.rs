use std::fmt;
use std::time::Duration;

use rust_decimal_macros::dec;
use strum::Display;
use time::OffsetDateTime;
use tracing::warn;

use crate::{
    constants::TIMESTAMP_FORMAT,
    events::{EventType, EventTypeOf},
    types::Commission,
    Event, Notional, Price, Quantity, StrategyId,
};

use super::Instrument;

#[derive(Clone)]
pub struct ExecutionOrder {
    pub id: u64,
    pub strategy_id: StrategyId,
    pub instrument: Instrument,
    pub side: Side,
    pub execution_type: ExecutionType,
    pub time_in_force: TimeInForce,
    pub last_fill_price: Price,
    pub last_fill_quantity: Quantity,
    pub last_fill_commission: Commission,
    pub total_avg_price: Price,
    pub total_quantity: Quantity,
    pub total_quantity_filled: Quantity,
    pub total_commission: Commission,
    pub status: ExecutionStatus,
    pub created_at: OffsetDateTime,
    pub last_updated_at: OffsetDateTime,
}

impl ExecutionOrder {
    pub fn new(
        event_time: OffsetDateTime,
        id: u64,
        strategy_id: StrategyId,
        instrument: Instrument,
        side: Side,
        execution_type: ExecutionType,
        time_in_force: TimeInForce,
        quantity: Quantity,
    ) -> Self {
        ExecutionOrder {
            id,
            strategy_id,
            instrument,
            side,
            execution_type,
            time_in_force,
            last_fill_price: dec!(0).into(),
            last_fill_quantity: dec!(0).into(),
            last_fill_commission: dec!(0).into(),
            total_avg_price: dec!(0).into(),
            total_quantity: quantity,
            total_quantity_filled: dec!(0).into(),
            total_commission: dec!(0).into(),
            status: ExecutionStatus::Pending,
            created_at: event_time,
            last_updated_at: event_time,
        }
    }

    pub fn new_market_order(
        id: u64,
        event_time: OffsetDateTime,
        strategy_id: StrategyId,
        instrument: Instrument,
        side: Side,
        quantity: Quantity,
    ) -> Self {
        Self::new(
            event_time,
            id,
            strategy_id,
            instrument,
            side,
            ExecutionType::Market,
            TimeInForce::Gtc,
            quantity,
        )
    }

    pub fn update(&mut self, event_time: OffsetDateTime, price: Price, quantity: Quantity, commission: Notional) {
        self.last_fill_price = price;
        self.last_fill_quantity = quantity;
        self.last_fill_commission = commission;

        self.total_avg_price = (self.total_avg_price * self.total_quantity_filled.abs() + price * quantity)
            / (self.total_quantity_filled.abs() + quantity);
        self.total_quantity_filled += quantity;
        self.total_commission += commission;
        self.last_updated_at = event_time;

        // Update the state
        match self.total_quantity_filled.abs() == self.total_quantity.abs() {
            true => self.status = ExecutionStatus::Filled,
            false => self.status = ExecutionStatus::PartiallyFilled,
        }
    }

    pub fn update_status(&mut self, event_time: OffsetDateTime, new_status: ExecutionStatus) {
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.last_updated_at = event_time;
        } else {
            warn!(
                "Invalid state transition from {} to {} for order {}",
                self.status, new_status, self.id
            );
        }
    }

    fn is_valid_transition(&self, new_status: &ExecutionStatus) -> bool {
        matches!(
            (&self.status, new_status),
            (ExecutionStatus::Pending, ExecutionStatus::Open)
                | (ExecutionStatus::Pending, ExecutionStatus::Cancelled)
                | (ExecutionStatus::Pending, ExecutionStatus::Rejected)
                | (ExecutionStatus::Open, ExecutionStatus::Cancelled)
                | (ExecutionStatus::Open, ExecutionStatus::Expired)
                | (ExecutionStatus::Open, ExecutionStatus::PartiallyFilled)
                | (ExecutionStatus::PartiallyFilled, ExecutionStatus::Cancelled)
                | (ExecutionStatus::PartiallyFilled, ExecutionStatus::Expired)
                | (ExecutionStatus::PartiallyFilled, ExecutionStatus::Filled)
        )
    }

    pub fn remaining_quantity(&self) -> Quantity {
        self.total_quantity - self.total_quantity_filled
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            ExecutionStatus::Pending | ExecutionStatus::Open | ExecutionStatus::PartiallyFilled
        )
    }

    pub fn has_fill(&self) -> bool {
        self.total_quantity_filled >= Quantity::ZERO
    }

    pub fn fill_time(&self) -> Option<Duration> {
        match self.is_active() {
            false => Some(Duration::from_millis(
                (self.last_updated_at - self.created_at).whole_milliseconds() as u64,
            )),
            true => None,
        }
    }

    pub fn notional(&self) -> Notional {
        self.total_avg_price * self.total_quantity_filled.abs()
    }

    pub fn value(&self) -> Notional {
        self.total_avg_price * self.total_quantity
    }
}

#[derive(Display, Clone)]
pub enum ExecutionType {
    Market,
    Limit(Price),
    TakeProfit(Price),
    Stop,
    StopLimit(Price),
    Algo(AlgoType),
}

#[derive(Display, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Display, Clone)]
pub enum AlgoType {
    Iceberg,
    Peg,
    Sniper,
    Twap,
    Vwap,
}

#[derive(Display, Clone)]
pub enum TimeInForce {
    Gtc,
    Ioc,
    Fok,
    Gtd(OffsetDateTime),
}

#[derive(Display, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

impl EventTypeOf for ExecutionOrder {
    fn event_type() -> EventType {
        EventType::Order
    }
}

impl TryFrom<Event> for ExecutionOrder {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Order(order) = event {
            Ok(order)
        } else {
            Err(())
        }
    }
}

impl From<ExecutionOrder> for Event {
    fn from(order: ExecutionOrder) -> Self {
        Event::Order(order)
    }
}

impl fmt::Display for ExecutionOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {} {} {}",
            self.last_updated_at
                .format(TIMESTAMP_FORMAT)
                .expect("Unable to format timestamp"),
            self.strategy_id,
            self.instrument,
            self.id,
            self.execution_type,
            self.total_quantity,
            self.total_quantity_filled,
            self.time_in_force,
            self.status
        )
    }
}
