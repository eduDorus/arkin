use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use sqlx::Type;
use strum::Display;
use time::UtcDateTime;
use tracing::error;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{types::Commission, Notional, Price, Quantity, VenueOrderId};

use super::{Instrument, MarketSide, Strategy};

pub type ExecutionOrderId = Uuid;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "execution_order_type", rename_all = "snake_case")]
pub enum ExecutionStrategyType {
    WideQuoter,
    Maker,
    Taker,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Type, PartialOrd, Ord)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "execution_order_status", rename_all = "snake_case")]
pub enum ExecutionOrderStatus {
    New,
    Placed,
    Rejected,
    PartiallyFilled,
    Filled,
    Cancelling,
    PartiallyFilledCancelled,
    Cancelled,
    PartiallyFilledExpired,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, TypedBuilder, Hash)]
#[builder(mutators(
    #[mutator(requires = [instrument])]
    pub fn set_price(&mut self, value: Price) {
        if value.is_zero() {
            self.price = Price::ZERO;  // Or handle as needed.
            return;
        }
        // Scale logic (adapted from your code).
        let scaling_factor = Decimal::ONE / self.instrument.tick_size;
        let scaled_price = value * scaling_factor;
        let rounded_scaled_price = scaled_price.round();
        let rounded_price = rounded_scaled_price * self.instrument.tick_size;
        self.price = rounded_price.round_dp(self.instrument.price_precision);
    }

    #[mutator(requires = [instrument])]
    pub fn set_quantity(&mut self, value: Quantity) {
        if value.is_zero() {
            self.quantity = Quantity::ZERO;
            return;
        }
        // Scale logic.
        let scaling_factor = Decimal::ONE / self.instrument.lot_size;
        let scaled_quantity = value * scaling_factor;
        let rounded_scaled_quantity = scaled_quantity.round();
        let round_quantity = rounded_scaled_quantity * self.instrument.lot_size;
        self.quantity = round_quantity.round_dp(self.instrument.quantity_precision);
    }
))]
pub struct ExecutionOrder {
    #[builder(default = Uuid::new_v4())]
    pub id: ExecutionOrderId,
    pub instrument: Arc<Instrument>,
    pub strategy: Option<Arc<Strategy>>,
    #[builder(default = vec![])]
    pub venue_order_ids: Vec<VenueOrderId>,
    pub exec_strategy_type: ExecutionStrategyType,
    pub side: MarketSide,
    #[builder(via_mutators(init = Price::ZERO))]
    pub price: Price,
    #[builder(via_mutators(init = Quantity::ZERO))]
    pub quantity: Quantity,
    #[builder(default = Price::ZERO)]
    pub fill_price: Price,
    #[builder(default = Quantity::ZERO)]
    pub filled_quantity: Quantity,
    #[builder(default = Notional::ZERO)]
    pub total_commission: Commission,
    #[builder(default = ExecutionOrderStatus::New)]
    pub status: ExecutionOrderStatus,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl ExecutionOrder {
    pub fn place(&mut self, event_time: UtcDateTime) {
        let new_status = ExecutionOrderStatus::Placed;
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
        } else {
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
        }
    }

    pub fn add_fill(&mut self, event_time: UtcDateTime, price: Price, quantity: Quantity, commission: Commission) {
        if !matches!(self.status, ExecutionOrderStatus::Placed | ExecutionOrderStatus::Cancelling) {
            error!("Cannot add fill in state {}", self.status);
            return;
        }
        self.fill_price =
            (self.fill_price * self.filled_quantity + price * quantity) / (self.filled_quantity + quantity);
        self.filled_quantity += quantity;
        self.total_commission += commission;
        self.updated = event_time;

        if self.remaining_quantity().is_zero() {
            let new_status = ExecutionOrderStatus::Filled;
            if self.is_valid_transition(&new_status) {
                self.status = new_status;
            } else {
                error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
            }
        }
    }

    pub fn cancel(&mut self, event_time: UtcDateTime) {
        let new_status = ExecutionOrderStatus::Cancelling;
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
        } else {
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
        }
    }

    pub fn expire(&mut self, event_time: UtcDateTime) {
        let new_status = if self.has_fill() {
            ExecutionOrderStatus::PartiallyFilledExpired
        } else {
            ExecutionOrderStatus::Expired
        };
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
        } else {
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
        }
    }

    pub fn reject(&mut self, event_time: UtcDateTime) {
        let new_status = ExecutionOrderStatus::Rejected;
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
        } else {
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
        }
    }

    pub fn finalize_terminate(&mut self, event_time: UtcDateTime) -> bool {
        let new_status = match self.status {
            ExecutionOrderStatus::Cancelling => {
                if self.remaining_quantity().is_zero() {
                    ExecutionOrderStatus::Filled
                } else if self.has_fill() {
                    ExecutionOrderStatus::PartiallyFilledCancelled
                } else {
                    ExecutionOrderStatus::Cancelled
                }
            }
            _ => {
                return false;
            }
        };
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
            return true;
        } else {
            error!("Invalid transition to {} from {}", new_status, self.status);
            return false;
        }
    }

    fn is_valid_transition(&self, new_status: &ExecutionOrderStatus) -> bool {
        matches!(
            (&self.status, new_status),
            (ExecutionOrderStatus::New, ExecutionOrderStatus::Placed)
                | (ExecutionOrderStatus::New, ExecutionOrderStatus::Rejected)
                | (ExecutionOrderStatus::New, ExecutionOrderStatus::Cancelling)
                | (ExecutionOrderStatus::Placed, ExecutionOrderStatus::PartiallyFilled)
                | (ExecutionOrderStatus::Placed, ExecutionOrderStatus::Filled)
                | (ExecutionOrderStatus::Placed, ExecutionOrderStatus::Cancelling)
                | (ExecutionOrderStatus::Placed, ExecutionOrderStatus::Expired)
                | (ExecutionOrderStatus::PartiallyFilled, ExecutionOrderStatus::Filled)
                | (ExecutionOrderStatus::Cancelling, ExecutionOrderStatus::PartiallyFilledCancelled)
                | (ExecutionOrderStatus::Cancelling, ExecutionOrderStatus::Filled)
                | (ExecutionOrderStatus::Cancelling, ExecutionOrderStatus::Cancelled)
        )
    }

    pub fn is_new(&self) -> bool {
        matches!(self.status, ExecutionOrderStatus::New)
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            ExecutionOrderStatus::Placed | ExecutionOrderStatus::PartiallyFilled
        )
    }

    pub fn is_terminating(&self) -> bool {
        matches!(self.status, ExecutionOrderStatus::Cancelling)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            ExecutionOrderStatus::Rejected
                | ExecutionOrderStatus::PartiallyFilledCancelled
                | ExecutionOrderStatus::PartiallyFilledExpired
                | ExecutionOrderStatus::Filled
                | ExecutionOrderStatus::Cancelled
                | ExecutionOrderStatus::Expired
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
            self.exec_strategy_type,
            self.price,
            self.quantity,
            self.total_value(),
            self.status
        )
    }
}
