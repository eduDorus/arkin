use std::{fmt, sync::Arc};

use sqlx::{FromRow, Type};
use strum::Display;
use time::OffsetDateTime;
use tracing::warn;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{types::Commission, Event, EventType, EventTypeOf, Notional, Price, Quantity};

use super::{Instrument, MarketSide, Portfolio, VenueOrderFill};

pub type ExecutionOrderId = Uuid;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "execution_order_type", rename_all = "snake_case")]
pub enum ExecutionOrderType {
    Maker,
    Taker,
    VWAP,
    TWAP,
    ALGO,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "execution_order_status", rename_all = "snake_case")]
pub enum ExecutionOrderStatus {
    New,
    InProgress,
    PartiallyFilled,
    PartiallyFilledCancelling,
    PartiallyFilledCancelled,
    Filled,
    Cancelling,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, TypedBuilder, FromRow)]

pub struct ExecutionOrder {
    #[builder(default = Uuid::new_v4())]
    pub id: ExecutionOrderId,
    pub portfolio: Arc<Portfolio>,
    pub instrument: Arc<Instrument>,
    pub order_type: ExecutionOrderType,
    pub side: MarketSide,
    #[builder(default = None)]
    pub price: Option<Price>,
    pub quantity: Quantity,
    #[builder(default = Price::ZERO)]
    pub fill_price: Price,
    #[builder(default = Quantity::ZERO)]
    pub filled_quantity: Quantity,
    #[builder(default = Notional::ZERO)]
    pub total_commission: Commission,
    #[builder(default = ExecutionOrderStatus::New)]
    pub status: ExecutionOrderStatus,
    #[builder(default = OffsetDateTime::now_utc())]
    pub created_at: OffsetDateTime,
    #[builder(default = OffsetDateTime::now_utc())]
    pub updated_at: OffsetDateTime,
}

impl ExecutionOrder {
    pub fn add_fill(&mut self, fill: VenueOrderFill) {
        self.fill_price = (self.fill_price * self.filled_quantity + fill.price * fill.quantity)
            / (self.filled_quantity + fill.quantity);
        self.filled_quantity += fill.quantity;
        self.total_commission += fill.commission;
        self.updated_at = fill.event_time;

        // Update the state
        match self.remaining_quantity().is_zero() {
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

    pub fn cancel(&mut self) {
        match self.status {
            ExecutionOrderStatus::New => self.status = ExecutionOrderStatus::Cancelled,
            ExecutionOrderStatus::InProgress => self.status = ExecutionOrderStatus::Cancelling,
            ExecutionOrderStatus::PartiallyFilled => self.status = ExecutionOrderStatus::PartiallyFilledCancelling,
            _ => warn!("Cannot cancel order in state {}", self.status),
        }
    }

    pub fn is_new(&self) -> bool {
        self.status == ExecutionOrderStatus::New
    }

    pub fn is_in_progress(&self) -> bool {
        matches!(
            self.status,
            ExecutionOrderStatus::InProgress | ExecutionOrderStatus::PartiallyFilled
        )
    }

    pub fn is_cancelling(&self) -> bool {
        matches!(
            self.status,
            ExecutionOrderStatus::Cancelling | ExecutionOrderStatus::PartiallyFilledCancelling
        )
    }

    pub fn is_cancelled(&self) -> bool {
        matches!(
            self.status,
            ExecutionOrderStatus::PartiallyFilledCancelled | ExecutionOrderStatus::Cancelled
        )
    }

    pub fn is_closed(&self) -> bool {
        matches!(
            self.status,
            ExecutionOrderStatus::PartiallyFilledCancelled
                | ExecutionOrderStatus::Cancelled
                | ExecutionOrderStatus::Filled
        )
    }

    fn is_valid_transition(&self, new_status: &ExecutionOrderStatus) -> bool {
        match (&self.status, new_status) {
            (ExecutionOrderStatus::New, ExecutionOrderStatus::InProgress)
            | (ExecutionOrderStatus::New, ExecutionOrderStatus::Cancelled)
            | (ExecutionOrderStatus::InProgress, ExecutionOrderStatus::PartiallyFilled)
            | (ExecutionOrderStatus::InProgress, ExecutionOrderStatus::Filled)
            | (ExecutionOrderStatus::InProgress, ExecutionOrderStatus::Cancelling)
            | (ExecutionOrderStatus::PartiallyFilled, ExecutionOrderStatus::PartiallyFilledCancelling)
            | (ExecutionOrderStatus::PartiallyFilled, ExecutionOrderStatus::Filled)
            | (ExecutionOrderStatus::PartiallyFilledCancelling, ExecutionOrderStatus::PartiallyFilledCancelled)
            | (ExecutionOrderStatus::Cancelling, ExecutionOrderStatus::Cancelled) => true,
            _ => false,
        }
    }

    pub fn remaining_quantity(&self) -> Quantity {
        self.quantity - self.filled_quantity
    }

    pub fn has_fill(&self) -> bool {
        self.filled_quantity > Quantity::ZERO
    }

    pub fn notional(&self) -> Notional {
        self.fill_price * self.filled_quantity
    }
}

impl EventTypeOf for ExecutionOrder {
    fn event_type() -> EventType {
        EventType::ExecutionOrderNew
    }
}

impl From<Arc<ExecutionOrder>> for Event {
    fn from(order: Arc<ExecutionOrder>) -> Self {
        Event::ExecutionOrderNew(order)
    }
}

impl fmt::Display for ExecutionOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ExecutionOrder: instrument={} order_type={} filled_quantity={} status={}",
            self.instrument, self.order_type, self.filled_quantity, self.status
        )
    }
}
