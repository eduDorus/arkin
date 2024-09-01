use std::fmt;

use strum::Display;
use time::OffsetDateTime;

use crate::{types::Commission, Notional, Price, Quantity, StrategyId};

use super::{Instrument, Side};

#[derive(Clone)]
pub struct Position {
    pub strategy_id: StrategyId,
    pub instrument: Instrument,
    pub side: PositionSide,
    pub avg_open_price: Price,
    pub avg_close_price: Price,
    pub quantity: Quantity,
    pub total_quantity: Quantity,
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
        side: Side,
        price: Price,
        quantity: Quantity,
        commission: Commission,
    ) -> Self {
        let mut position = Self {
            strategy_id,
            instrument,
            side: match side {
                Side::Buy => PositionSide::Long,
                Side::Sell => PositionSide::Short,
            },
            avg_open_price: Price::default(),
            avg_close_price: Price::default(),
            quantity: Quantity::default(),
            total_quantity: Quantity::default(),
            realized_pnl: Commission::default(),
            commission: Commission::default(),
            status: PositionStatus::Open,
            created_at: event_time,
            last_updated_at: event_time,
        };
        position.update(event_time, side, price, quantity, commission);
        position
    }

    pub fn update(
        &mut self,
        event_time: OffsetDateTime,
        side: Side,
        price: Price,
        quantity: Quantity,
        commission: Notional,
    ) -> Option<Quantity> {
        let remaining = match (&self.side, &side) {
            (PositionSide::Long, Side::Buy) | (PositionSide::Short, Side::Sell) => {
                self.add_to_position(price, quantity);
                None
            }
            (PositionSide::Long, Side::Sell) | (PositionSide::Short, Side::Buy) => {
                self.reduce_or_close_position(price, quantity)
            }
        };
        // If there is a remaining quantity we only want to add the fraction of the commission
        // that is proportional to the filled quantity
        if remaining.is_some() {
            self.commission += commission * ((quantity - remaining.unwrap()) / quantity);
        } else {
            self.commission += commission;
        };
        self.total_quantity += quantity;
        self.last_updated_at = event_time;
        remaining
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
