use std::fmt;

use super::{ExecutionOrder, ExecutionStatus, Instrument, Notional, OrderSide, Price, Quantity, StrategyId};
use strum::Display;
use time::OffsetDateTime;
use tracing::warn;

#[derive(Clone)]
pub struct Position {
    pub strategy_id: StrategyId,
    pub instrument: Instrument,
    pub side: PositionSide,
    pub avg_open_price: Price,
    pub avg_close_price: Price,
    pub quantity: Quantity,
    pub realized_pnl: Notional,
    pub commission: Notional,
    pub status: PositionStatus,
    pub created_at: OffsetDateTime,
    pub last_updated_at: OffsetDateTime,
}

#[derive(Clone, Display, Copy, PartialEq, Eq, Debug)]
pub enum PositionSide {
    Long,
    Short,
}

#[derive(Clone, Display, Copy, PartialEq, Eq, Debug)]
pub enum PositionStatus {
    Open,
    Closed,
}

impl Position {
    pub fn new(
        event_time: OffsetDateTime,
        strategy_id: StrategyId,
        instrument: Instrument,
        side: PositionSide,
        price: Price,
        quantity: Quantity,
        commission: Notional,
    ) -> Self {
        let mut position = Self {
            strategy_id,
            instrument,
            side,
            avg_open_price: Price::default(),
            avg_close_price: Price::default(),
            quantity: Quantity::default(),
            realized_pnl: Notional::default(),
            commission: Notional::default(),
            status: PositionStatus::Open,
            created_at: event_time,
            last_updated_at: event_time,
        };
        let orderside = match side {
            PositionSide::Long => OrderSide::Buy,
            PositionSide::Short => OrderSide::Sell,
        };
        position.update(event_time, orderside, price, quantity, commission);
        position
    }

    pub fn update(
        &mut self,
        event_time: OffsetDateTime,
        side: OrderSide,
        price: Price,
        quantity: Quantity,
        commission: Notional,
    ) -> Option<Quantity> {
        let remaining = match (&self.side, &side) {
            (PositionSide::Long, OrderSide::Buy) | (PositionSide::Short, OrderSide::Sell) => {
                self.add_to_position(price, quantity);
                None
            }
            (PositionSide::Long, OrderSide::Sell) | (PositionSide::Short, OrderSide::Buy) => {
                self.reduce_or_close_position(price, quantity)
            }
        };
        self.commission += commission;
        self.last_updated_at = event_time;
        remaining
    }

    pub fn update_with_order(&mut self, order: &ExecutionOrder) -> Option<Quantity> {
        if self.is_valid_execution(&order.status) {
            self.update(
                order.last_updated_at,
                order.side,
                order.last_fill_price,
                order.last_fill_quantity,
                order.last_fill_commission,
            )
        } else {
            warn!(
                "Ignoring order {} with status {} for position {}",
                order.id, order.status, self.instrument
            );
            None
        }
    }

    fn is_valid_execution(&self, status: &ExecutionStatus) -> bool {
        matches!(status, ExecutionStatus::Filled | ExecutionStatus::PartiallyFilled)
    }

    fn add_to_position(&mut self, price: Price, quantity: Quantity) {
        self.avg_open_price = (self.avg_open_price * self.quantity + price * quantity) / (self.quantity + quantity);
        self.quantity += quantity;
    }

    fn reduce_or_close_position(&mut self, price: Price, quantity: Quantity) -> Option<Quantity> {
        let fillable_quantity = self.quantity.min(quantity);
        match self.side {
            PositionSide::Long => self.realized_pnl += Price::from(price - self.avg_open_price) * fillable_quantity,
            PositionSide::Short => self.realized_pnl += Price::from(self.avg_open_price - price) * fillable_quantity,
        }
        self.avg_close_price = (self.avg_close_price * self.quantity + price * fillable_quantity) / (fillable_quantity);
        self.quantity -= fillable_quantity;

        if self.quantity.is_zero() {
            self.status = PositionStatus::Closed;
        }
        if fillable_quantity < quantity {
            return Some(quantity - fillable_quantity);
        } else {
            return None;
        }
    }
}

impl From<ExecutionOrder> for Position {
    fn from(order: ExecutionOrder) -> Self {
        let side = match order.side {
            OrderSide::Buy => PositionSide::Long,
            OrderSide::Sell => PositionSide::Short,
        };
        Self {
            strategy_id: order.strategy_id,
            instrument: order.instrument,
            side,
            avg_open_price: order.total_avg_price,
            avg_close_price: Price::default(),
            quantity: order.last_fill_quantity,
            realized_pnl: Notional::default(),
            commission: order.total_commission, // Initialize with the commission from the order
            status: PositionStatus::Open,
            created_at: order.last_updated_at,
            last_updated_at: order.last_updated_at,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} avg open: {} avg close: {} quantity: {} pnl: {} commission: {} status: {}",
            self.strategy_id,
            self.instrument,
            self.side,
            self.avg_open_price,
            self.avg_close_price,
            self.quantity,
            self.realized_pnl,
            self.commission,
            self.status
        )
    }
}
