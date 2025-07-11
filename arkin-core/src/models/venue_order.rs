use std::{fmt, sync::Arc};

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
    StopLimit,
    StopMarket,
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
    PartiallyFilledCancelled,
    PartiallyFilledExpired,
    Filled,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, TypedBuilder)]
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
    pub price: Price,
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
    pub created_at: UtcDateTime,
    pub updated_at: UtcDateTime,
}

impl VenueOrder {
    pub fn add_fill(&mut self, event_time: UtcDateTime, price: Price, quantity: Quantity, commission: Commission) {
        self.last_fill_price = price;
        self.last_fill_quantity = quantity;
        self.last_fill_commission = commission;

        self.filled_price =
            (self.filled_price * self.filled_quantity + price * quantity) / (self.filled_quantity + quantity);
        self.filled_quantity += quantity;
        self.commission += commission;
        self.updated_at = event_time;

        self.status = match self.remaining_quantity().is_zero() {
            false => VenueOrderStatus::PartiallyFilled,
            true => VenueOrderStatus::Filled,
        };
    }

    pub fn update_status(&mut self, new_status: VenueOrderStatus, time: UtcDateTime) {
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
            self.updated_at = time
        } else {
            error!(
                "Invalid state transition from {} to {} for order {}",
                self.status, new_status, self.id
            );
        }
    }

    pub fn cancelled(&mut self) {
        match self.status {
            VenueOrderStatus::New => self.status = VenueOrderStatus::Cancelled,
            VenueOrderStatus::Placed => self.status = VenueOrderStatus::Cancelled,
            VenueOrderStatus::PartiallyFilled => self.status = VenueOrderStatus::PartiallyFilledCancelled,
            _ => error!("Cannot cancel order in state {}", self.status),
        }
    }

    pub fn expired(&mut self) {
        match self.status {
            VenueOrderStatus::Placed => self.status = VenueOrderStatus::Expired,
            VenueOrderStatus::PartiallyFilled => self.status = VenueOrderStatus::PartiallyFilledExpired,
            _ => error!("Cannot expire order in state {}", self.status),
        }
    }

    pub fn update_commision_asset(&mut self, asset: Arc<Asset>) {
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
            (VenueOrderStatus::New, VenueOrderStatus::Placed)
                | (VenueOrderStatus::New, VenueOrderStatus::Inflight)
                | (VenueOrderStatus::New, VenueOrderStatus::Cancelled)
                | (VenueOrderStatus::New, VenueOrderStatus::Rejected)
                | (VenueOrderStatus::Inflight, VenueOrderStatus::Placed)
                | (VenueOrderStatus::Inflight, VenueOrderStatus::Rejected)
                | (VenueOrderStatus::Placed, VenueOrderStatus::PartiallyFilled)
                | (VenueOrderStatus::Placed, VenueOrderStatus::Filled)
                | (VenueOrderStatus::Placed, VenueOrderStatus::Cancelled)
                | (VenueOrderStatus::Placed, VenueOrderStatus::Expired)
                | (VenueOrderStatus::PartiallyFilled, VenueOrderStatus::Filled)
                | (VenueOrderStatus::PartiallyFilled, VenueOrderStatus::PartiallyFilledCancelled)
                | (VenueOrderStatus::PartiallyFilled, VenueOrderStatus::PartiallyFilledExpired)
        )
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, VenueOrderStatus::Placed | VenueOrderStatus::PartiallyFilled)
    }

    pub fn is_finalized(&self) -> bool {
        matches!(
            self.status,
            VenueOrderStatus::Rejected
                | VenueOrderStatus::Cancelled
                | VenueOrderStatus::Expired
                | VenueOrderStatus::Filled
                | VenueOrderStatus::PartiallyFilledCancelled
                | VenueOrderStatus::PartiallyFilledExpired
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
