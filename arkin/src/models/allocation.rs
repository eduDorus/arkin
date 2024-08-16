use super::{Event, EventType, EventTypeOf, Instrument, Notional, Price, Quantity, StrategyId, Weight};
use rust_decimal::Decimal;
use std::{fmt, time::Duration};
use strum::Display;
use time::OffsetDateTime;

#[derive(Clone)]
pub struct Allocation {
    pub event_time: OffsetDateTime,
    pub strategy_id: StrategyId,
    pub instrument: Instrument,
    pub current_allocation: Weight,
    pub current_price: Price,
    pub current_quantity: Quantity,
    pub target_allocation: Weight,
    pub target_price: Price,
    pub target_quantity: Quantity,
    pub value_at_risk: Notional,
    pub expected_shortfall: Notional,
    pub beta: Decimal,
}

impl EventTypeOf for Allocation {
    fn event_type() -> EventType {
        EventType::Allocation
    }
}

impl TryFrom<Event> for Allocation {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::Allocation(allocation) = event {
            Ok(allocation)
        } else {
            Err(())
        }
    }
}

impl From<Allocation> for Event {
    fn from(allocation: Allocation) -> Self {
        Event::Allocation(allocation)
    }
}

impl fmt::Display for Allocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {}",
            self.event_time, self.strategy_id, self.instrument, self.current_allocation, self.target_allocation,
        )
    }
}

#[derive(Clone)]
pub struct ExecutionOrder {
    pub event_time: OffsetDateTime,
    pub strategy_id: StrategyId,
    pub instrument: Instrument,
    pub order_id: u64,
    pub execution_type: ExecutionType,
    pub side: OrderSide,
    pub avg_price: Price,
    pub quantity: Quantity,
    pub quantity_filled: Quantity,
    pub time_in_force: TimeInForce,
    pub commission: Notional,
    pub status: ExecutionStatus,
    pub created_at: OffsetDateTime,
    pub last_updated_at: OffsetDateTime,
}

impl ExecutionOrder {
    pub fn remaining_quantity(&self) -> Quantity {
        self.quantity - self.quantity_filled
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            ExecutionStatus::Pending | ExecutionStatus::Open | ExecutionStatus::PartiallyFilled
        )
    }

    pub fn fill_time(&self) -> Option<Duration> {
        match self.status {
            ExecutionStatus::Filled | ExecutionStatus::PartiallyFilled | ExecutionStatus::PartiallyFilledCancelled => {
                Some(Duration::from_millis(
                    (self.last_updated_at - self.event_time).whole_milliseconds() as u64,
                ))
            }
            _ => None,
        }
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

#[derive(Display, Clone)]
pub enum OrderSide {
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
    PartiallyFilledCancelled,
    Filled,
    Cancelled,
    Rejected,
}

impl EventTypeOf for ExecutionOrder {
    fn event_type() -> EventType {
        EventType::ExecutionOrder
    }
}

impl TryFrom<Event> for ExecutionOrder {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::ExecutionOrder(order) = event {
            Ok(order)
        } else {
            Err(())
        }
    }
}

impl fmt::Display for ExecutionOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} {} {} {}",
            self.event_time,
            self.strategy_id,
            self.instrument,
            self.order_id,
            self.execution_type,
            self.quantity,
            self.quantity_filled,
            self.time_in_force,
            self.status
        )
    }
}
