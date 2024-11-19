use std::{fmt, sync::Arc};

use derive_builder::Builder;
use rust_decimal::Decimal;
use strum::Display;
use tracing::warn;
use uuid::Uuid;

use crate::{
    events::{EventType, EventTypeOf},
    types::Commission,
    Event, Notional, Price, Quantity,
};

use super::{Fill, Instrument, MarketSide};

pub type ExecutionOrderId = Uuid;

#[derive(Debug, Display, Clone, PartialEq, Eq)]
pub enum ExecutionOrderStrategy {
    Market,
    Limit(Price),
    WideQuoting(Decimal),
}

#[derive(Debug, Display, Clone, PartialEq, Eq)]
pub enum ExecutionOrderStatus {
    New,
    InProgress,
    PartiallyFilled,
    PartiallyFilledCancelled,
    Filled,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
#[builder(setter(into))]
pub struct ExecutionOrder {
    #[builder(default = Uuid::new_v4())]
    pub id: ExecutionOrderId,
    pub instrument: Arc<Instrument>,
    pub side: MarketSide,
    pub execution_type: ExecutionOrderStrategy,
    #[builder(default = Price::ZERO)]
    pub avg_fill_price: Price,
    pub quantity: Quantity,
    #[builder(default = Quantity::ZERO)]
    pub filled_quantity: Quantity,
    #[builder(default = Notional::ZERO)]
    pub total_commission: Commission,
    #[builder(default = ExecutionOrderStatus::New)]
    pub status: ExecutionOrderStatus,
}

impl ExecutionOrder {
    pub fn add_fill(&mut self, fill: Fill) {
        self.avg_fill_price = (self.avg_fill_price * self.filled_quantity + fill.price * fill.quantity)
            / (self.filled_quantity + fill.quantity);
        self.filled_quantity += fill.quantity;
        self.total_commission += fill.commission;

        // Update the state
        match self.filled_quantity == self.quantity {
            true => self.status = ExecutionOrderStatus::Filled,
            false => self.status = ExecutionOrderStatus::PartiallyFilled,
        }
    }

    pub fn update_status(&mut self, new_status: ExecutionOrderStatus) {
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
        } else {
            warn!(
                "Invalid state transition from {} to {} for order {}",
                self.status, new_status, self.id
            );
        }
    }

    fn is_valid_transition(&self, new_status: &ExecutionOrderStatus) -> bool {
        matches!((&self.status, new_status), |(
            ExecutionOrderStatus::New,
            ExecutionOrderStatus::Cancelled,
        )| (
            ExecutionOrderStatus::New,
            ExecutionOrderStatus::InProgress,
        ) | (
            ExecutionOrderStatus::PartiallyFilled,
            ExecutionOrderStatus::PartiallyFilledCancelled
        ))
    }

    pub fn remaining_quantity(&self) -> Quantity {
        self.quantity - self.filled_quantity
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            ExecutionOrderStatus::InProgress | ExecutionOrderStatus::PartiallyFilled
        )
    }

    pub fn has_fill(&self) -> bool {
        self.filled_quantity > Quantity::ZERO
    }

    pub fn notional(&self) -> Notional {
        self.avg_fill_price * self.filled_quantity
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
            "{} {} price: {}/{} quantity: {}/{} {}",
            self.instrument,
            self.side,
            self.execution_type,
            self.avg_fill_price,
            self.filled_quantity,
            self.quantity,
            self.status
        )
    }
}
