use std::time::Duration;
use std::{fmt, sync::Arc};

use strum::Display;
use time::OffsetDateTime;
use tracing::warn;
use uuid::Uuid;

use crate::{
    constants::TIMESTAMP_FORMAT,
    events::{EventType, EventTypeOf},
    types::Commission,
    Event, Notional, Price, Quantity,
};

use super::{Account, ExecutionOrder, Instrument, MarketSide, Strategy};

#[derive(Debug, Display, Clone)]
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
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
}

#[derive(Debug, Clone)]
pub struct VenueOrder {
    pub id: Uuid,
    pub account: Account,
    pub instrument: Arc<Instrument>,
    pub strategy: Strategy,
    pub execution_order: ExecutionOrder,
    pub venue_order_id: u64,
    pub side: MarketSide,
    pub order_type: VenueOrderType,
    pub time_in_force: VenueOrderTimeInForce,
    pub price: Price,
    pub avg_fill_price: Price,
    pub quantity: Quantity,
    pub filled_quantity: Quantity,
    pub total_commission: Commission,
    pub status: VenueOrderStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl VenueOrder {
    pub fn new_market(
        account: Account,
        instrument: Arc<Instrument>,
        strategy: Strategy,
        execution_order: ExecutionOrder,
        side: MarketSide,
        quantity: Quantity,
        created_at: OffsetDateTime,
    ) -> Self {
        VenueOrder {
            id: Uuid::new_v4(),
            account,
            instrument,
            strategy,
            execution_order,
            venue_order_id: 0,
            side,
            order_type: VenueOrderType::Market,
            time_in_force: VenueOrderTimeInForce::Gtc,
            price: Price::ZERO,
            avg_fill_price: Price::ZERO,
            quantity,
            filled_quantity: Quantity::ZERO,
            total_commission: Commission::ZERO,
            status: VenueOrderStatus::New,
            created_at,
            updated_at: created_at,
        }
    }

    pub fn new_limit(
        account: Account,
        instrument: Arc<Instrument>,
        strategy: Strategy,
        execution_order: ExecutionOrder,
        side: MarketSide,
        price: Price,
        quantity: Quantity,
        created_at: OffsetDateTime,
    ) -> Self {
        VenueOrder {
            id: Uuid::new_v4(),
            account,
            instrument,
            strategy,
            execution_order,
            venue_order_id: 0,
            side,
            order_type: VenueOrderType::Limit,
            time_in_force: VenueOrderTimeInForce::Gtc,
            price,
            avg_fill_price: Price::ZERO,
            quantity,
            filled_quantity: Quantity::ZERO,
            total_commission: Commission::ZERO,
            status: VenueOrderStatus::New,
            created_at,
            updated_at: created_at,
        }
    }

    pub fn update(&mut self, event_time: OffsetDateTime, price: Price, quantity: Quantity, commission: Commission) {
        self.avg_fill_price =
            (self.avg_fill_price * self.filled_quantity + price * quantity) / (self.filled_quantity + quantity);
        self.filled_quantity += quantity;
        self.total_commission += commission;
        self.updated_at = event_time;

        // Update the state
        match self.filled_quantity == self.quantity {
            true => self.status = VenueOrderStatus::Filled,
            false => self.status = VenueOrderStatus::PartiallyFilled,
        }
    }

    pub fn update_status(&mut self, event_time: OffsetDateTime, new_status: VenueOrderStatus) {
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

    fn is_valid_transition(&self, new_status: &VenueOrderStatus) -> bool {
        matches!(
            (&self.status, new_status),
            (VenueOrderStatus::New, VenueOrderStatus::Open)
                | (VenueOrderStatus::New, VenueOrderStatus::Rejected)
                | (VenueOrderStatus::New, VenueOrderStatus::Cancelled)
                | (VenueOrderStatus::Open, VenueOrderStatus::PartiallyFilled)
                | (VenueOrderStatus::Open, VenueOrderStatus::Filled)
                | (VenueOrderStatus::Open, VenueOrderStatus::Cancelled)
                | (VenueOrderStatus::PartiallyFilled, VenueOrderStatus::Filled)
                | (VenueOrderStatus::PartiallyFilled, VenueOrderStatus::Cancelled)
        )
    }

    pub fn remaining_quantity(&self) -> Quantity {
        self.quantity - self.filled_quantity
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, VenueOrderStatus::Open | VenueOrderStatus::PartiallyFilled)
    }

    pub fn has_fill(&self) -> bool {
        self.filled_quantity > Quantity::ZERO
    }

    pub fn fill_time(&self) -> Option<Duration> {
        match self.has_fill() {
            false => Some(Duration::from_millis(
                (self.updated_at - self.created_at).whole_milliseconds() as u64
            )),
            true => None,
        }
    }

    pub fn notional(&self) -> Notional {
        self.avg_fill_price * self.filled_quantity
    }
}

impl EventTypeOf for VenueOrder {
    fn event_type() -> EventType {
        EventType::VenueOrder
    }
}

impl TryFrom<Event> for VenueOrder {
    type Error = ();

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        if let Event::VenueOrder(order) = event {
            Ok(order)
        } else {
            Err(())
        }
    }
}

impl From<VenueOrder> for Event {
    fn from(order: VenueOrder) -> Self {
        Event::VenueOrder(order)
    }
}

impl fmt::Display for VenueOrder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} {} {} {} price: {}/{} quantity: {}/{} {}",
            self.updated_at.format(TIMESTAMP_FORMAT).expect("Unable to format timestamp"),
            self.account,
            self.instrument,
            self.strategy,
            self.side,
            self.order_type,
            self.avg_fill_price,
            self.price,
            self.filled_quantity,
            self.quantity,
            self.status
        )
    }
}
