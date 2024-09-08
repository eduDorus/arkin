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
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} {} avg open: {} avg close: {} pnl: {} commission: {} status: {}",
            self.strategy_id,
            self.instrument,
            self.side,
            self.avg_open_price.round_dp(2),
            self.avg_close_price.round_dp(2),
            // self.quantity.round_dp(4),
            self.realized_pnl.round_dp(2),
            self.commission.round_dp(2),
            self.status
        )
    }
}
