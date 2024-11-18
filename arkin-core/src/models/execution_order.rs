use std::time::Duration;
use std::{fmt, sync::Arc};

use strum::Display;
use time::OffsetDateTime;
use tracing::warn;
use uuid::Uuid;

use crate::{
    constants::TIMESTAMP_FORMAT,
    events::{EventType, EventTypeOf},
    types::Commission,
    Event, Notional, Price, Quantity,
};

use super::{Account, Instrument, MarketSide, Signal, Strategy};

pub type OrderId = Uuid;

#[derive(Debug, Display, Clone)]
pub enum ExecutionOrderStrategy {
    Maker,
    Taker,
    VWAP,
}

#[derive(Debug, Display, Clone, PartialEq, Eq)]
pub enum ExecutionOrderStatus {
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct ExecutionOrder {
    pub id: OrderId,
    pub account: Account,
    pub instrument: Arc<Instrument>,
    pub strategy: Strategy,
    pub signal: Signal,
    pub side: MarketSide,
    pub execution_type: ExecutionOrderStrategy,
    pub current_price: Price,
    pub avg_fill_price: Price,
    pub quantity: Quantity,
    pub filled_quantity: Quantity,
    pub total_commission: Commission,
    pub status: ExecutionOrderStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl ExecutionOrder {
    pub fn new(
        account: Account,
        instrument: Arc<Instrument>,
        strategy: Strategy,
        signal: Signal,
        side: MarketSide,
        execution_type: ExecutionOrderStrategy,
        current_price: Price,
        quantity: Quantity,
        created_at: OffsetDateTime,
    ) -> Self {
        ExecutionOrder {
            id: Uuid::new_v4(),
            account,
            instrument,
            strategy,
            signal,
            side,
            execution_type,
            current_price,
            avg_fill_price: Price::ZERO,
            quantity,
            filled_quantity: Quantity::ZERO,
            total_commission: Notional::ZERO,
            status: ExecutionOrderStatus::Open,
            created_at,
            updated_at: created_at,
        }
    }

    pub fn update(&mut self, event_time: OffsetDateTime, price: Price, quantity: Quantity, commission: Commission) {
        self.avg_fill_price =
            (self.avg_fill_price * self.filled_quantity + price * quantity) / (self.filled_quantity + quantity);
        self.filled_quantity += quantity;
        self.total_commission += commission;
        self.updated_at = event_time;

        // Update the state
        match self.filled_quantity == self.quantity {
            true => self.status = ExecutionOrderStatus::Filled,
            false => self.status = ExecutionOrderStatus::PartiallyFilled,
        }
    }

    pub fn update_status(&mut self, event_time: OffsetDateTime, new_status: ExecutionOrderStatus) {
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated_at = event_time;
        } else {
            warn!(
                "Invalid state transition from {} to {} for order {}",
                self.status, new_status, self.id
            );
        }
    }

    fn is_valid_transition(&self, new_status: &ExecutionOrderStatus) -> bool {
        matches!((&self.status, new_status), |(
            ExecutionOrderStatus::Open,
            ExecutionOrderStatus::Cancelled,
        )| (
            ExecutionOrderStatus::Open,
            ExecutionOrderStatus::PartiallyFilled
        ) | (
            ExecutionOrderStatus::PartiallyFilled,
            ExecutionOrderStatus::Cancelled
        ) | (
            ExecutionOrderStatus::PartiallyFilled,
            ExecutionOrderStatus::Filled
        ))
    }

    pub fn remaining_quantity(&self) -> Quantity {
        self.quantity - self.filled_quantity
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, ExecutionOrderStatus::Open | ExecutionOrderStatus::PartiallyFilled)
    }

    pub fn has_fill(&self) -> bool {
        self.filled_quantity > Quantity::ZERO
    }

    pub fn fill_time(&self) -> Option<Duration> {
        match self.has_fill() {
            false => Some(Duration::from_millis(
                (self.updated_at - self.created_at).whole_milliseconds() as u64
            )),
            true => None,
        }
    }

    pub fn notional(&self) -> Notional {
        self.avg_fill_price * self.filled_quantity
    }

    pub fn value(&self) -> Notional {
        self.current_price * self.quantity
    }
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

impl From<ExecutionOrder> for Event {
    fn from(order: ExecutionOrder) -> Self {
        Event::ExecutionOrder(order)
    }
}

impl fmt::Display for ExecutionOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} price: {}/{} quantity: {}/{} {}",
            self.updated_at.format(TIMESTAMP_FORMAT).expect("Unable to format timestamp"),
            self.account,
            self.instrument,
            self.strategy,
            self.side,
            self.execution_type,
            self.avg_fill_price,
            self.current_price,
            self.filled_quantity,
            self.quantity,
            self.status
        )
    }
}
