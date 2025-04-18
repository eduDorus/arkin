use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use sqlx::Type;
use strum::Display;
use time::OffsetDateTime;
use tracing::warn;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{types::Commission, Notional, Price, Quantity};

use super::{Instrument, MarketSide, Strategy};

pub type ExecutionOrderId = Uuid;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "execution_order_type", rename_all = "snake_case")]
pub enum ExecutionOrderType {
    Maker,
    Taker,
    VWAP,
    TWAP,
    ALGO,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Type)]
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

#[derive(Debug, Clone, PartialEq, Eq, TypedBuilder, Hash)]
pub struct ExecutionOrder {
    #[builder(default = Uuid::new_v4())]
    pub id: ExecutionOrderId,
    pub event_time: OffsetDateTime,
    pub strategy: Option<Arc<Strategy>>,
    pub instrument: Arc<Instrument>,
    pub order_type: ExecutionOrderType,
    pub side: MarketSide,
    pub price: Price,
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
    pub updated_at: OffsetDateTime,
}

impl ExecutionOrder {
    pub fn add_fill(&mut self, event_time: OffsetDateTime, price: Price, quantity: Quantity, commission: Commission) {
        self.fill_price =
            (self.fill_price * self.filled_quantity + price * quantity) / (self.filled_quantity + quantity);
        self.filled_quantity += quantity;
        self.total_commission += commission;
        self.updated_at = event_time;

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

    pub fn total_value(&self) -> Decimal {
        self.price * self.quantity * self.instrument.contract_size
    }
}

impl fmt::Display for ExecutionOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "instrument={} side={} order_type={} price={} quantity={} total_value={} status={}",
            self.instrument,
            self.side,
            self.order_type,
            self.price,
            self.quantity,
            self.total_value(),
            self.status
        )
    }
}
