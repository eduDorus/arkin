use std::{fmt, sync::Arc};

use derive_builder::Builder;
use rust_decimal::Decimal;
use strum::Display;
use time::OffsetDateTime;
use tracing::warn;
use uuid::Uuid;

use crate::{types::Commission, EventTypeOf, Notional, Price, Quantity, Event, EventType, VenueOrderFill};

use super::{Instrument, MarketSide};

pub type ExecutionOrderId = Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionOrderStrategy {
    Market(Market),
    Limit(Limit),
    // WideQuoting(WideQuoting),
}

impl fmt::Display for ExecutionOrderStrategy {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExecutionOrderStrategy::Market(o) => write!(f, "MKT side={} quantity={}", o.side, o.quantity),
            ExecutionOrderStrategy::Limit(o) => {
                write!(f, "LMT side={} price={} quantity={}", o.side, o.price, o.quantity)
            } // ExecutionOrderStrategy::WideQuoting(o) => write!(f, "WideQuoting: {}", o),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
#[builder(setter(into))]
pub struct Market {
    pub side: MarketSide,
    pub quantity: Quantity,
    #[builder(default = false)]
    pub split: bool,
    #[builder(default = false)]
    pub vwap: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
#[builder(setter(into))]
pub struct Limit {
    pub side: MarketSide,
    pub quantity: Quantity,
    pub price: Price,
    pub time_in_force: Option<OffsetDateTime>,
    pub split: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
#[builder(setter(into))]
pub struct WideQuoting {
    pub spread_from_mid: Decimal,
    pub requote_price_move_pct: Decimal,
}

#[derive(Debug, Display, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq, Builder)]
#[builder(setter(into))]
pub struct ExecutionOrder {
    #[builder(default = Uuid::new_v4())]
    pub id: ExecutionOrderId,
    #[builder(default = OffsetDateTime::now_utc())]
    pub event_time: OffsetDateTime,
    pub instrument: Arc<Instrument>,
    pub execution_type: ExecutionOrderStrategy,
    #[builder(default = Price::ZERO)]
    pub avg_fill_price: Price,
    #[builder(default = Quantity::ZERO)]
    pub filled_quantity: Quantity,
    #[builder(default = Notional::ZERO)]
    pub total_commission: Commission,
    #[builder(default = ExecutionOrderStatus::New)]
    pub status: ExecutionOrderStatus,
}

impl ExecutionOrder {
    pub fn add_fill(&mut self, fill: VenueOrderFill) {
        self.avg_fill_price = (self.avg_fill_price * self.filled_quantity + fill.price * fill.quantity)
            / (self.filled_quantity + fill.quantity);
        self.filled_quantity += fill.quantity;
        self.total_commission += fill.commission;

        // Update the state
        match self.filled_quantity == self.quantity() {
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

    pub fn quantity(&self) -> Quantity {
        match &self.execution_type {
            ExecutionOrderStrategy::Market(o) => o.quantity,
            ExecutionOrderStrategy::Limit(o) => o.quantity,
        }
    }

    pub fn side(&self) -> MarketSide {
        match &self.execution_type {
            ExecutionOrderStrategy::Market(o) => o.side,
            ExecutionOrderStrategy::Limit(o) => o.side,
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
        self.quantity() - self.filled_quantity
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

impl From<ExecutionOrder> for Event {
    fn from(order: ExecutionOrder) -> Self {
        Event::ExecutionOrder(order)
    }
}

impl fmt::Display for ExecutionOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "ExecutionOrder: instrument={} execution_type={} filled_quantity={} status={}",
            self.instrument, self.execution_type, self.filled_quantity, self.status
        )
    }
}
