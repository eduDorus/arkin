use std::{fmt, sync::Arc};

use derive_builder::Builder;
use strum::Display;
use time::OffsetDateTime;
use tracing::error;
use uuid::Uuid;

use crate::{types::Commission, Event, Notional, Price, Quantity, UpdateEventType, VenueOrderFill};

use super::{ExecutionOrderId, Instrument, MarketSide};

pub type VenueOrderId = Uuid;

#[derive(Debug, Display, Clone, PartialEq, Eq)]
pub enum VenueOrderType {
    Market,
    Limit,
}

#[derive(Debug, Display, Clone)]
pub enum VenueOrderTimeInForce {
    Gtc,
    Ioc,
    Fok,
    Gtd(OffsetDateTime),
}

#[derive(Debug, Display, Clone, PartialEq, Eq)]
pub enum VenueOrderStatus {
    New,
    Placed,
    PartiallyFilled,
    PartiallyFilledCancelling,
    PartiallyFilledCancelled,
    Filled,
    Cancelling,
    Cancelled,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Builder)]
pub struct VenueOrder {
    #[builder(default = Uuid::new_v4())]
    pub id: VenueOrderId,
    pub execution_order_id: ExecutionOrderId,
    pub instrument: Arc<Instrument>,
    pub side: MarketSide,
    pub order_type: VenueOrderType,
    #[builder(default = VenueOrderTimeInForce::Gtc)]
    pub time_in_force: VenueOrderTimeInForce,
    pub price: Option<Price>,
    #[builder(default = Price::ZERO)]
    pub avg_fill_price: Price,
    pub quantity: Quantity,
    #[builder(default = Quantity::ZERO)]
    pub filled_quantity: Quantity,
    #[builder(default = Commission::ZERO)]
    pub total_commission: Commission,
    #[builder(default = VenueOrderStatus::New)]
    pub status: VenueOrderStatus,
}

impl VenueOrder {
    pub fn new_market(
        execution_order_id: ExecutionOrderId,
        instrument: Arc<Instrument>,
        side: MarketSide,
        quantity: Quantity,
    ) -> Self {
        VenueOrder {
            id: Uuid::new_v4(),
            execution_order_id,
            instrument,
            side,
            order_type: VenueOrderType::Market,
            time_in_force: VenueOrderTimeInForce::Gtc,
            price: None,
            avg_fill_price: Price::ZERO,
            quantity,
            filled_quantity: Quantity::ZERO,
            total_commission: Commission::ZERO,
            status: VenueOrderStatus::New,
        }
    }

    pub fn new_limit(instrument: Arc<Instrument>, side: MarketSide, price: Price, quantity: Quantity) -> Self {
        VenueOrder {
            id: Uuid::new_v4(),
            execution_order_id: Uuid::new_v4(),
            instrument,
            side,
            order_type: VenueOrderType::Limit,
            time_in_force: VenueOrderTimeInForce::Gtc,
            price: Some(price),
            avg_fill_price: Price::ZERO,
            quantity,
            filled_quantity: Quantity::ZERO,
            total_commission: Commission::ZERO,
            status: VenueOrderStatus::New,
        }
    }

    pub fn update(&mut self, price: Price, quantity: Quantity, commission: Commission) {
        self.avg_fill_price =
            (self.avg_fill_price * self.filled_quantity + price * quantity) / (self.filled_quantity + quantity);
        self.filled_quantity += quantity;
        self.total_commission += commission;

        // Update the state
        match self.filled_quantity == self.quantity {
            true => self.status = VenueOrderStatus::Filled,
            false => self.status = VenueOrderStatus::PartiallyFilled,
        }
    }

    pub fn add_fill(&mut self, fill: VenueOrderFill) {
        self.avg_fill_price = (self.avg_fill_price * self.filled_quantity + fill.price * fill.quantity)
            / (self.filled_quantity + fill.quantity);
        self.filled_quantity += fill.quantity;
        self.total_commission += fill.commission;
        self.status = match self.filled_quantity == self.quantity {
            true => VenueOrderStatus::Filled,
            false => VenueOrderStatus::PartiallyFilled,
        };
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
            VenueOrderStatus::New => self.status = VenueOrderStatus::Cancelled,
            VenueOrderStatus::Placed => self.status = VenueOrderStatus::Cancelling,
            VenueOrderStatus::PartiallyFilled => self.status = VenueOrderStatus::PartiallyFilledCancelling,
            _ => error!("Cannot cancel order in state {}", self.status),
        }
    }

    pub fn ack_cancel(&mut self) {
        match self.status {
            VenueOrderStatus::Cancelling => self.status = VenueOrderStatus::Cancelled,
            VenueOrderStatus::PartiallyFilledCancelling => self.status = VenueOrderStatus::PartiallyFilledCancelled,
            _ => error!("Cannot ack cancel order in state {}", self.status),
        }
    }

    fn is_valid_transition(&self, new_status: &VenueOrderStatus) -> bool {
        matches!(
            (&self.status, new_status),
            (VenueOrderStatus::New, VenueOrderStatus::Placed)
                | (VenueOrderStatus::New, VenueOrderStatus::Rejected)
                | (VenueOrderStatus::New, VenueOrderStatus::Cancelled)
                | (VenueOrderStatus::Placed, VenueOrderStatus::PartiallyFilled)
                | (VenueOrderStatus::Placed, VenueOrderStatus::Filled)
                | (VenueOrderStatus::Placed, VenueOrderStatus::Cancelling)
                | (VenueOrderStatus::PartiallyFilled, VenueOrderStatus::Filled)
                | (VenueOrderStatus::PartiallyFilled, VenueOrderStatus::Cancelling)
                | (
                    VenueOrderStatus::PartiallyFilledCancelling,
                    VenueOrderStatus::PartiallyFilledCancelled
                )
                | (VenueOrderStatus::Cancelling, VenueOrderStatus::Cancelled)
        )
    }

    pub fn remaining_quantity(&self) -> Quantity {
        self.quantity - self.filled_quantity
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, VenueOrderStatus::Placed | VenueOrderStatus::PartiallyFilled)
    }

    pub fn is_cancelling(&self) -> bool {
        matches!(
            self.status,
            VenueOrderStatus::Cancelling | VenueOrderStatus::PartiallyFilledCancelling
        )
    }

    pub fn is_finalized(&self) -> bool {
        matches!(
            self.status,
            VenueOrderStatus::PartiallyFilledCancelled | VenueOrderStatus::Cancelled | VenueOrderStatus::Filled
        )
    }

    pub fn has_fill(&self) -> bool {
        self.filled_quantity > Quantity::ZERO
    }

    pub fn notional(&self) -> Notional {
        self.avg_fill_price * self.filled_quantity
    }
}

impl Event for VenueOrder {
    fn event_type() -> UpdateEventType {
        UpdateEventType::VenueOrder
    }
}

impl fmt::Display for VenueOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "id: {} exec_id: {} {} {} {} price: {}/{} quantity: {}/{} {}",
            self.id,
            self.execution_order_id,
            self.instrument,
            self.side,
            self.order_type,
            self.avg_fill_price,
            self.price.unwrap_or(Price::ZERO),
            self.filled_quantity,
            self.quantity,
            self.status
        )
    }
}
