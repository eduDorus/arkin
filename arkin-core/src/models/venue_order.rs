use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use sqlx::Type;
use strum::Display;
use time::UtcDateTime;
use tracing::error;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::models::{Asset, Instrument, MarketSide, Strategy};
use crate::ExecutionOrderId;
use crate::{types::Commission, Price, Quantity};

pub type VenueOrderId = Uuid;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "venue_order_type", rename_all = "snake_case")]
pub enum VenueOrderType {
    Market,
    Limit,
    StopMarket,
    StopLimit,
    TakeProfit,
    TakeProfitMarket,
    TrailingStopMarket,
}

#[derive(Debug, Display, Clone, Copy, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "venue_order_time_in_force", rename_all = "snake_case")]
pub enum VenueOrderTimeInForce {
    Gtc,
    Ioc,
    Fok,
    Gtx,
    Gtd,
}

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Type, PartialOrd, Ord)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "venue_order_status", rename_all = "snake_case")]
pub enum VenueOrderStatus {
    New,
    Inflight,
    Placed,
    Rejected,
    PartiallyFilled,
    Filled,
    Cancelling,
    Cancelled,
    PartiallyFilledCancelled,
    Expired,
    PartiallyFilledExpired,
}

#[derive(Debug, Clone, TypedBuilder)]
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
pub struct VenueOrder {
    #[builder(default = Uuid::new_v4())]
    pub id: VenueOrderId,
    #[builder(default = None)]
    pub execution_order_id: Option<ExecutionOrderId>,
    pub instrument: Arc<Instrument>,
    pub strategy: Option<Arc<Strategy>>,
    pub side: MarketSide,
    #[builder(default = VenueOrderType::Market)]
    pub order_type: VenueOrderType,
    #[builder(default = VenueOrderTimeInForce::Gtc)]
    pub time_in_force: VenueOrderTimeInForce,
    // Initial order price and quantity
    #[builder(via_mutators(init = Price::ZERO))]
    pub price: Price,
    #[builder(via_mutators(init = Quantity::ZERO))]
    pub quantity: Quantity,
    // Last fill price and quantity
    #[builder(default = Price::ZERO)]
    pub last_fill_price: Price,
    #[builder(default = Quantity::ZERO)]
    pub last_fill_quantity: Quantity,
    #[builder(default = Commission::ZERO)]
    pub last_fill_commission: Commission,
    // Average fill price and total filled quantity
    #[builder(default = Price::ZERO)]
    pub filled_price: Price,
    #[builder(default = Quantity::ZERO)]
    pub filled_quantity: Quantity,
    #[builder(default = None)]
    pub commission_asset: Option<Arc<Asset>>,
    #[builder(default = Commission::ZERO)]
    pub commission: Commission,
    #[builder(default = VenueOrderStatus::New)]
    pub status: VenueOrderStatus,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl VenueOrder {
    pub fn set_inflight(&mut self, event_time: UtcDateTime) {
        let new_status = VenueOrderStatus::Inflight;
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
        } else {
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
        }
    }

    pub fn place(&mut self, event_time: UtcDateTime) {
        let new_status = VenueOrderStatus::Placed;
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
        } else {
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
        }
    }

    pub fn add_fill(&mut self, event_time: UtcDateTime, price: Price, quantity: Quantity, commission: Commission) {
        if !matches!(
            self.status,
            VenueOrderStatus::New
                | VenueOrderStatus::Inflight
                | VenueOrderStatus::Placed
                | VenueOrderStatus::PartiallyFilled
                | VenueOrderStatus::Cancelling
        ) {
            error!("Cannot add fill in state {}", self.status);
            return;
        }
        self.last_fill_price = price;
        self.last_fill_quantity = quantity;
        self.last_fill_commission = commission;

        self.filled_price =
            (self.filled_price * self.filled_quantity + price * quantity) / (self.filled_quantity + quantity);
        self.filled_quantity += quantity;
        self.commission += commission;
        self.updated = event_time;

        if self.remaining_quantity().is_zero() {
            let new_status = VenueOrderStatus::Filled;
            if self.is_valid_transition(&new_status) {
                self.status = new_status;
            } else {
                error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
            }
        } else {
            let new_status = VenueOrderStatus::PartiallyFilled;
            if self.is_valid_transition(&new_status) {
                self.status = new_status;
            } else {
                error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
            }
        }
    }

    pub fn cancel(&mut self, event_time: UtcDateTime) {
        let new_status = VenueOrderStatus::Cancelling;
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
        } else {
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
        }
    }

    pub fn expire(&mut self, event_time: UtcDateTime) {
        let new_status = VenueOrderStatus::Expired;
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
        } else {
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
        }
    }

    pub fn reject(&mut self, event_time: UtcDateTime) {
        let new_status = VenueOrderStatus::Rejected;
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated = event_time;
        } else {
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
        }
    }

    pub fn finalize_terminate(&mut self, event_time: UtcDateTime) -> bool {
        let new_status = match self.status {
            VenueOrderStatus::Cancelling => {
                if self.remaining_quantity().is_zero() {
                    VenueOrderStatus::Filled
                } else if self.has_fill() {
                    VenueOrderStatus::PartiallyFilledCancelled
                } else {
                    VenueOrderStatus::Cancelled
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
            error!("Invalid transition to {} from {} for {}", new_status, self.status, self.id);
            return false;
        }
    }

    pub fn update_commission_asset(&mut self, asset: Arc<Asset>) {
        self.commission_asset = Some(asset);
    }

    pub fn remaining_quantity(&self) -> Quantity {
        self.quantity - self.filled_quantity
    }

    pub fn has_fill(&self) -> bool {
        self.filled_quantity > Quantity::ZERO
    }

    pub fn total_value(&self) -> Price {
        self.price * self.quantity * self.instrument.contract_size
    }

    fn is_valid_transition(&self, new_status: &VenueOrderStatus) -> bool {
        matches!(
            (&self.status, new_status),
            (VenueOrderStatus::New, VenueOrderStatus::Inflight)
                | (VenueOrderStatus::New, VenueOrderStatus::Placed)
                | (VenueOrderStatus::New, VenueOrderStatus::Cancelled)
                | (VenueOrderStatus::New, VenueOrderStatus::PartiallyFilled)
                | (VenueOrderStatus::Inflight, VenueOrderStatus::Placed)
                | (VenueOrderStatus::Inflight, VenueOrderStatus::Rejected)
                | (VenueOrderStatus::Placed, VenueOrderStatus::PartiallyFilled)
                | (VenueOrderStatus::Placed, VenueOrderStatus::Filled)
                | (VenueOrderStatus::Placed, VenueOrderStatus::Cancelling)
                | (VenueOrderStatus::PartiallyFilled, VenueOrderStatus::PartiallyFilled)
                | (VenueOrderStatus::PartiallyFilled, VenueOrderStatus::Cancelling)
                | (VenueOrderStatus::PartiallyFilled, VenueOrderStatus::Filled)
                | (VenueOrderStatus::Cancelling, VenueOrderStatus::Cancelled)
                | (VenueOrderStatus::Cancelling, VenueOrderStatus::PartiallyFilledCancelled)
                | (VenueOrderStatus::Cancelling, VenueOrderStatus::Filled)
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            VenueOrderStatus::New | VenueOrderStatus::Inflight | VenueOrderStatus::Placed
        )
    }

    pub fn is_terminating(&self) -> bool {
        matches!(self.status, VenueOrderStatus::Cancelling)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            VenueOrderStatus::Filled
                | VenueOrderStatus::Cancelled
                | VenueOrderStatus::PartiallyFilledCancelled
                | VenueOrderStatus::Expired
                | VenueOrderStatus::PartiallyFilledExpired
                | VenueOrderStatus::Rejected
        )
    }
}

impl fmt::Display for VenueOrder {
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
