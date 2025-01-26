use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use sqlx::prelude::Type;
use strum::Display;
use time::OffsetDateTime;
use tracing::info;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{types::Commission, Event, EventType, EventTypeOf, Notional, Price, Quantity, VenueOrderFill};

use super::{Instrument, MarketSide, Portfolio, Strategy};

#[derive(Clone, Display, Copy, PartialEq, Eq, Debug, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "position_side", rename_all = "snake_case")]
pub enum PositionSide {
    Long,
    Short,
    // Both, // Quantity decides so the position can be both long and short
}

impl From<MarketSide> for PositionSide {
    fn from(side: MarketSide) -> Self {
        match side {
            MarketSide::Buy => PositionSide::Long,
            MarketSide::Sell => PositionSide::Short,
        }
    }
}

#[derive(Clone, Display, Copy, PartialEq, Eq, Debug, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "position_status", rename_all = "snake_case")]
pub enum PositionStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct Position {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    // pub portfolio: Arc<Portfolio>,
    pub strategy: Arc<Strategy>,
    pub instrument: Arc<Instrument>,
    pub side: PositionSide,
    pub open_price: Price,
    pub open_quantity: Quantity,
    #[builder(default = Decimal::ZERO)]
    pub close_price: Price,
    #[builder(default = Decimal::ZERO)]
    pub close_quantity: Quantity,
    pub last_price: Price,
    #[builder(default = Decimal::ZERO)]
    pub realized_pnl: Notional,
    #[builder(default = Decimal::ZERO)]
    pub total_commission: Commission,
    #[builder(default = PositionStatus::Open)]
    pub status: PositionStatus,
    #[builder(default = OffsetDateTime::now_utc())]
    pub created_at: OffsetDateTime,
    #[builder(default = OffsetDateTime::now_utc())]
    pub updated_at: OffsetDateTime,
}

pub enum Action {
    Increase,
    Decrease,
}

impl Position {
    pub fn update_price(&mut self, price: Price) {
        self.last_price = price;
    }

    pub fn add_fill(&mut self, fill: VenueOrderFill) -> Option<VenueOrderFill> {
        info!("Updating position with fill: {}", fill);
        let action = match (self.side, fill.side) {
            (PositionSide::Long, MarketSide::Buy) => Action::Increase,
            (PositionSide::Long, MarketSide::Sell) => Action::Decrease,
            (PositionSide::Short, MarketSide::Buy) => Action::Decrease,
            (PositionSide::Short, MarketSide::Sell) => Action::Increase,
        };

        match action {
            Action::Increase => {
                self.increase_position(fill);
                None
            }
            Action::Decrease => {
                let max_fill_quantity = fill.quantity.min(self.open_quantity);
                let remaining_fill_quantity = fill.quantity - max_fill_quantity;

                if remaining_fill_quantity.is_zero() {
                    self.decrease_position(fill);
                    None
                } else {
                    let mut current_fill = fill.clone();
                    current_fill.quantity = max_fill_quantity;
                    current_fill.commission = ((current_fill.commission / fill.quantity) * max_fill_quantity)
                        .round_dp(self.instrument.price_precision);
                    self.decrease_position(current_fill);
                    let mut remaining_fill = fill.clone();
                    remaining_fill.quantity = remaining_fill_quantity;
                    remaining_fill.commission = ((remaining_fill.commission / fill.quantity) * remaining_fill_quantity)
                        .round_dp(self.instrument.price_precision);

                    Some(remaining_fill)
                }
            }
        }
    }

    fn increase_position(&mut self, fill: VenueOrderFill) {
        info!("Increasing position: {}", self);
        self.open_price = (self.open_price * self.open_quantity)
            + (fill.price * fill.quantity) / (self.open_quantity + fill.quantity);
        self.open_quantity += fill.quantity;
        self.total_commission += fill.commission;
        self.updated_at = fill.event_time;
        info!("Updated position: {}", self);
    }

    fn decrease_position(&mut self, fill: VenueOrderFill) {
        info!("Decreasing position: {}", self);
        self.close_price = (self.close_price * self.close_quantity)
            + (fill.price * fill.quantity) / (self.close_quantity + fill.quantity);
        self.close_quantity += fill.quantity;
        self.total_commission += fill.commission;
        let realized_pnl = match self.side {
            PositionSide::Long => fill.price * fill.quantity - self.open_price * fill.quantity,
            PositionSide::Short => self.open_price * fill.quantity - fill.price * fill.quantity,
        };
        self.realized_pnl += realized_pnl.round_dp(self.instrument.price_precision);
        self.updated_at = fill.event_time;

        if self.quantity().is_zero() {
            self.status = PositionStatus::Closed;
            info!("Closed position: {}", self);
        } else {
            info!("Updated position: {}", self);
        }
    }

    /// The total value of your current position based on the latest market prices.
    pub fn market_value(&self) -> Notional {
        self.last_price * self.quantity_with_side() * self.instrument.contract_size
    }

    /// The total value of the underlying asset that a financial derivative represents. It provides a measure of the total exposure.
    pub fn notional_value(&self) -> Notional {
        self.last_price * self.quantity() * self.instrument.contract_size
    }

    pub fn quantity(&self) -> Quantity {
        self.open_quantity - self.close_quantity
    }

    pub fn quantity_with_side(&self) -> Quantity {
        match self.side {
            PositionSide::Long => self.quantity(),
            PositionSide::Short => -self.quantity(),
        }
    }

    pub fn unrealized_pnl(&self) -> Notional {
        match self.side {
            PositionSide::Long => (self.last_price - self.open_price) * self.quantity(),
            PositionSide::Short => (self.open_price - self.last_price) * self.quantity(),
        }
    }

    pub fn return_pct(&self) -> Decimal {
        if self.close_price.is_zero() {
            Decimal::ZERO
        } else {
            match self.side {
                PositionSide::Long => (self.close_price - self.open_price) / self.open_price,
                PositionSide::Short => (self.open_price - self.close_price) / self.open_price,
            }
        }
    }

    pub fn is_open(&self) -> bool {
        self.status == PositionStatus::Open
    }

    pub fn is_closed(&self) -> bool {
        self.status == PositionStatus::Closed
    }

    pub fn is_profitable(&self) -> bool {
        self.realized_pnl > Decimal::ZERO
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Position {}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.created_at.cmp(&other.created_at)
    }
}

impl EventTypeOf for Position {
    fn event_type() -> EventType {
        EventType::Position
    }
}

impl From<Arc<Position>> for Event {
    fn from(position: Arc<Position>) -> Self {
        Event::Position(position)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "instrument={} side={} open_price={} close_price={} quantity={} realized_pnl={} unrealized_pnl={} total_commission={}",
            self.instrument,
            self.side,
            self.open_price,
            self.close_price,
            self.quantity(),
            self.realized_pnl,
            self.unrealized_pnl(),
            self.total_commission,
        )
    }
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct PositionUpdate {
    pub event_time: OffsetDateTime,
    pub portfolio: Arc<Portfolio>,
    pub instrument: Arc<Instrument>,
    pub entry_price: Price,
    pub quantity: Quantity,
    pub realized_pnl: Decimal,
    pub unrealized_pnl: Decimal,
    pub position_side: PositionSide,
}

impl PositionUpdate {
    // TODO: This is only for perpetual swaps (For short you still post collateral)
    pub fn market_value(&self) -> Decimal {
        self.entry_price * self.quantity * self.instrument.contract_size
    }

    pub fn notional_value(&self) -> Decimal {
        self.entry_price * self.quantity.abs() * self.instrument.contract_size
    }
}

impl EventTypeOf for PositionUpdate {
    fn event_type() -> EventType {
        EventType::PositionUpdate
    }
}

impl From<Arc<PositionUpdate>> for Event {
    fn from(update: Arc<PositionUpdate>) -> Self {
        Event::PositionUpdate(update)
    }
}

impl From<PositionUpdate> for Event {
    fn from(update: PositionUpdate) -> Self {
        Event::PositionUpdate(Arc::new(update))
    }
}

impl fmt::Display for PositionUpdate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "instrument={} side={} entry_price={} quantity={} realized_pnl={} unrealized_pnl={}",
            self.instrument,
            self.position_side,
            self.entry_price,
            self.quantity,
            self.realized_pnl,
            self.unrealized_pnl,
        )
    }
}
