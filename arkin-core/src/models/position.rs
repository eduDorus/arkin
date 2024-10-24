use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use strum::Display;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{types::Commission, Notional, Price, Quantity};

use super::{Account, Instrument, MarketSide, Strategy};

#[derive(Clone)]
pub struct Position {
    pub id: Uuid,
    pub account: Account,
    pub instrument: Arc<Instrument>,
    pub strategy: Strategy,
    pub side: PositionSide,
    pub avg_open_price: Price,
    pub avg_close_price: Price,
    pub quantity: Quantity,
    pub realized_pnl: Notional,
    pub commission: Notional,
    pub status: PositionStatus,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
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
        account: Account,
        strategy: Strategy,
        instrument: Arc<Instrument>,
        side: MarketSide,
        price: Price,
        quantity: Quantity,
        commission: Commission,
        event_time: OffsetDateTime,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            account,
            instrument,
            strategy,
            side: match side {
                MarketSide::Buy => PositionSide::Long,
                MarketSide::Sell => PositionSide::Short,
            },
            avg_open_price: price,
            avg_close_price: Decimal::new(18, 2),
            quantity,
            realized_pnl: Decimal::new(18, 2),
            commission,
            status: PositionStatus::Open,
            created_at: event_time,
            updated_at: event_time,
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
            "{} {} {} {} avg open: {} avg close: {} pnl: {} commission: {} return: {}",
            self.account,
            self.strategy,
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
