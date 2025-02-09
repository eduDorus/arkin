use std::{fmt, sync::Arc};

use sqlx::Type;
use strum::Display;
use time::OffsetDateTime;
use tracing::error;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{types::Commission, Event, EventType, EventTypeOf, Price, Quantity};

use super::{Asset, ExecutionOrder, ExecutionOrderType, Instrument, MarketSide, Portfolio, VenueOrderFill};

pub type VenueOrderId = Uuid;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "venue_order_type", rename_all = "snake_case")]
pub enum VenueOrderType {
    Market,
    Limit,
    Stop,
    StopMarket,
    TakeProfit,
    TakeProfitMarket,
    TrailingStopMarket,
    Liquidation,
}

impl From<ExecutionOrderType> for VenueOrderType {
    fn from(order_type: ExecutionOrderType) -> Self {
        match order_type {
            ExecutionOrderType::Maker => VenueOrderType::Limit,
            ExecutionOrderType::Taker => VenueOrderType::Market,
            ExecutionOrderType::VWAP => unimplemented!("VWAP not supported"),
            ExecutionOrderType::TWAP => unimplemented!("TWAP not supported"),
            ExecutionOrderType::ALGO => unimplemented!("ALGO not supported"),
        }
    }
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

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "venue_order_status", rename_all = "snake_case")]
pub enum VenueOrderStatus {
    New,
    Placed,
    PartiallyFilled,
    PartiallyFilledCanceled,
    PartiallyFilledExpired,
    Filled,
    Canceled,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct VenueOrder {
    #[builder(default = Uuid::new_v4())]
    pub id: VenueOrderId,
    pub portfolio: Arc<Portfolio>,
    pub instrument: Arc<Instrument>,
    pub side: MarketSide,
    pub order_type: VenueOrderType,
    #[builder(default = VenueOrderTimeInForce::Gtc)]
    pub time_in_force: VenueOrderTimeInForce,
    pub price: Price,
    pub quantity: Quantity,
    #[builder(default = Price::ZERO)]
    pub fill_price: Price,
    #[builder(default = Quantity::ZERO)]
    pub filled_quantity: Quantity,
    #[builder(default = Commission::ZERO)]
    pub total_commission: Commission,
    #[builder(default = VenueOrderStatus::New)]
    pub status: VenueOrderStatus,
    #[builder(default = OffsetDateTime::now_utc())]
    pub created_at: OffsetDateTime,
    #[builder(default = OffsetDateTime::now_utc())]
    pub updated_at: OffsetDateTime,
}

impl VenueOrder {
    pub fn add_fill(&mut self, fill: Arc<VenueOrderFill>) {
        self.fill_price = (self.fill_price * self.filled_quantity + fill.price * fill.quantity)
            / (self.filled_quantity + fill.quantity);
        self.filled_quantity += fill.quantity;
        self.total_commission += fill.commission;
        self.status = match self.filled_quantity == self.quantity {
            true => VenueOrderStatus::Filled,
            false => VenueOrderStatus::PartiallyFilled,
        };
        self.updated_at = fill.event_time;
    }

    pub fn update_status(&mut self, new_status: VenueOrderStatus) {
        if self.is_valid_transition(&new_status) {
            self.status = new_status;
        } else {
            error!(
                "Invalid state transition from {} to {} for order {}",
                self.status, new_status, self.id
            );
        }
    }

    pub fn cancel(&mut self) {
        match self.status {
            VenueOrderStatus::New => self.status = VenueOrderStatus::Canceled,
            VenueOrderStatus::Placed => self.status = VenueOrderStatus::Canceled,
            VenueOrderStatus::PartiallyFilled => self.status = VenueOrderStatus::PartiallyFilledCanceled,
            _ => error!("Cannot cancel order in state {}", self.status),
        }
    }

    pub fn remaining_quantity(&self) -> Quantity {
        self.quantity - self.filled_quantity
    }

    fn is_valid_transition(&self, new_status: &VenueOrderStatus) -> bool {
        matches!(
            (&self.status, new_status),
            (VenueOrderStatus::New, VenueOrderStatus::Placed)
                | (VenueOrderStatus::New, VenueOrderStatus::Rejected)
                | (VenueOrderStatus::New, VenueOrderStatus::Canceled)
                | (VenueOrderStatus::Placed, VenueOrderStatus::PartiallyFilled)
                | (VenueOrderStatus::Placed, VenueOrderStatus::Filled)
                | (VenueOrderStatus::Placed, VenueOrderStatus::Canceled)
                | (VenueOrderStatus::Placed, VenueOrderStatus::Expired)
                | (VenueOrderStatus::PartiallyFilled, VenueOrderStatus::Filled)
                | (VenueOrderStatus::PartiallyFilled, VenueOrderStatus::PartiallyFilledCanceled)
                | (VenueOrderStatus::PartiallyFilled, VenueOrderStatus::PartiallyFilledExpired)
        )
    }

    pub fn is_new(&self) -> bool {
        self.status == VenueOrderStatus::New
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, VenueOrderStatus::Placed | VenueOrderStatus::PartiallyFilled)
    }

    pub fn is_finalized(&self) -> bool {
        matches!(
            self.status,
            VenueOrderStatus::PartiallyFilledCanceled
                | VenueOrderStatus::PartiallyFilledExpired
                | VenueOrderStatus::Filled
                | VenueOrderStatus::Canceled
                | VenueOrderStatus::Rejected
                | VenueOrderStatus::Expired
        )
    }

    pub fn has_fill(&self) -> bool {
        self.filled_quantity > Quantity::ZERO
    }

    pub fn total_value(&self) -> Price {
        self.price * self.quantity * self.instrument.contract_size
    }
}

impl From<ExecutionOrder> for VenueOrder {
    fn from(order: ExecutionOrder) -> Self {
        Self {
            id: order.id,
            portfolio: order.portfolio,
            instrument: order.instrument,
            side: order.side,
            order_type: order.order_type.into(),
            time_in_force: VenueOrderTimeInForce::Gtc,
            price: order.price,
            quantity: order.quantity,
            fill_price: Price::ZERO,
            filled_quantity: Quantity::ZERO,
            total_commission: Commission::ZERO,
            status: VenueOrderStatus::New,
            created_at: order.created_at,
            updated_at: order.updated_at,
        }
    }
}

impl EventTypeOf for VenueOrder {
    fn event_type() -> EventType {
        EventType::VenueOrder
    }
}

impl From<Arc<VenueOrder>> for Event {
    fn from(order: Arc<VenueOrder>) -> Self {
        Event::VenueOrder(order)
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

#[derive(Debug, Clone, TypedBuilder)]
pub struct VenueOrderUpdate {
    pub event_time: OffsetDateTime,
    pub portfolio: Arc<Portfolio>,
    pub instrument: Arc<Instrument>,
    pub order_id: String,
    pub venue_order_id: i64,
    pub side: MarketSide,
    pub order_type: VenueOrderType,
    pub time_in_force: VenueOrderTimeInForce,
    pub price: Price,
    pub quantity: Quantity,
    pub fill_price: Price,
    pub fill_quantity: Quantity,
    pub last_fill_price: Price,
    pub last_fill_quantity: Quantity,
    pub status: VenueOrderStatus,
    pub commission_asset: Option<Arc<Asset>>,
    pub commission: Commission,
}

impl VenueOrderUpdate {
    pub fn total_value(&self) -> Price {
        self.price * self.quantity * self.instrument.contract_size
    }
}

impl EventTypeOf for VenueOrderUpdate {
    fn event_type() -> EventType {
        EventType::VenueOrderUpdate
    }
}

impl From<VenueOrderUpdate> for Event {
    fn from(update: VenueOrderUpdate) -> Self {
        Event::VenueOrderUpdate(Arc::new(update))
    }
}

impl From<Arc<VenueOrderUpdate>> for Event {
    fn from(update: Arc<VenueOrderUpdate>) -> Self {
        Event::VenueOrderUpdate(update)
    }
}

impl fmt::Display for VenueOrderUpdate {
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
