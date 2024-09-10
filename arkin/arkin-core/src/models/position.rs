use std::fmt;

use rust_decimal::Decimal;
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
    pub trade_volume: Notional,
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
        Self {
            strategy_id,
            instrument,
            side: match side {
                Side::Buy => PositionSide::Long,
                Side::Sell => PositionSide::Short,
            },
            avg_open_price: price,
            avg_close_price: Decimal::new(18, 2),
            quantity,
            trade_volume: Decimal::new(18, 4),
            realized_pnl: Decimal::new(18, 2),
            commission,
            status: PositionStatus::Open,
            created_at: event_time,
            last_updated_at: event_time,
        }
    }

    pub fn market_value(&self, price: Price) -> Notional {
        price * self.quantity
    }

    pub fn quantity_with_side(&self) -> Quantity {
        match self.side {
            PositionSide::Long => self.quantity,
            PositionSide::Short => -self.quantity,
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

    pub fn return_pct(&self) -> Decimal {
        match self.side {
            PositionSide::Long => (self.avg_close_price - self.avg_open_price) / self.avg_open_price,
            PositionSide::Short => (self.avg_open_price - self.avg_close_price) / self.avg_open_price,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} avg open: {} avg close: {} pnl: {} commission: {} return: {}",
            self.strategy_id,
            self.instrument,
            self.side,
            self.avg_open_price.round_dp(2),
            self.avg_close_price.round_dp(2),
            // self.quantity.round_dp(4),
            self.realized_pnl.round_dp(2),
            self.commission.round_dp(2),
            self.return_pct().round_dp(4),
        )
    }
}
