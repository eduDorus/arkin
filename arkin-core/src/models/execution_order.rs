use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use sqlx::Type;
use strum::Display;
use time::OffsetDateTime;
use tracing::warn;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{types::Commission, Notional, Price, Quantity, VenueOrderId};

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

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Type, PartialOrd, Ord)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "execution_order_status", rename_all = "snake_case")]
pub enum ExecutionOrderStatus {
    New,
    Active,
    Completed,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, TypedBuilder, Hash)]
pub struct ExecutionOrder {
    #[builder(default = Uuid::new_v4())]
    pub id: ExecutionOrderId,
    pub instrument: Arc<Instrument>,
    pub strategy: Option<Arc<Strategy>>,
    #[builder(default = vec![])]
    pub venue_order_ids: Vec<VenueOrderId>,
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
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl ExecutionOrder {
    pub fn add_fill(&mut self, event_time: OffsetDateTime, price: Price, quantity: Quantity, commission: Commission) {
        self.fill_price =
            (self.fill_price * self.filled_quantity + price * quantity) / (self.filled_quantity + quantity);
        self.filled_quantity += quantity;
        self.total_commission += commission;
        self.updated_at = event_time;

        if self.remaining_quantity().is_zero() {
            self.status = ExecutionOrderStatus::Completed;
        }
    }

    pub fn update_status(&mut self, new_status: ExecutionOrderStatus, event_time: OffsetDateTime) {
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

    pub fn cancel(&mut self) {
        match self.status {
            ExecutionOrderStatus::New => self.status = ExecutionOrderStatus::Cancelled,
            ExecutionOrderStatus::Active => self.status = ExecutionOrderStatus::Cancelled,
            _ => warn!("Cannot cancel order in state {}", self.status),
        }
    }

    fn is_valid_transition(&self, new_status: &ExecutionOrderStatus) -> bool {
        matches!(
            (&self.status, new_status),
            (ExecutionOrderStatus::New, ExecutionOrderStatus::Active)
                | (ExecutionOrderStatus::New, ExecutionOrderStatus::Cancelled)
                | (ExecutionOrderStatus::Active, ExecutionOrderStatus::Completed)
                | (ExecutionOrderStatus::Active, ExecutionOrderStatus::Cancelled)
                | (ExecutionOrderStatus::Active, ExecutionOrderStatus::Expired)
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, ExecutionOrderStatus::Active)
    }

    pub fn is_finalized(&self) -> bool {
        matches!(
            self.status,
            ExecutionOrderStatus::Completed | ExecutionOrderStatus::Cancelled | ExecutionOrderStatus::Expired
        )
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
