use std::{fmt, sync::Arc};

use derive_builder::Builder;
use rust_decimal::Decimal;
use strum::Display;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{types::Commission, Notional, Price, Quantity};

use super::{Fill, Instrument, MarketSide};

#[derive(Clone, Display, Copy, PartialEq, Eq, Debug)]
pub enum PositionSide {
    Long,
    Short,
}

impl From<MarketSide> for PositionSide {
    fn from(side: MarketSide) -> Self {
        match side {
            MarketSide::Buy => PositionSide::Long,
            MarketSide::Sell => PositionSide::Short,
        }
    }
}

#[derive(Clone, Display, Copy, PartialEq, Eq, Debug)]
pub enum PositionStatus {
    Open,
    Closed,
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct Position {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub instrument: Arc<Instrument>,
    pub side: PositionSide,
    pub avg_open_price: Price,
    pub avg_close_price: Option<Price>,
    pub quantity: Quantity,
    #[builder(default = Decimal::ZERO)]
    pub realized_pnl: Notional,
    #[builder(default = Decimal::ZERO)]
    pub commission: Commission,
    #[builder(default = PositionStatus::Open)]
    pub status: PositionStatus,
    #[builder(default = OffsetDateTime::now_utc())]
    pub created_at: OffsetDateTime,
    #[builder(default = OffsetDateTime::now_utc())]
    pub updated_at: OffsetDateTime,
}

impl Position {
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
        if let Some(avg_close_price) = self.avg_close_price {
            match self.side {
                PositionSide::Long => (avg_close_price - self.avg_open_price) / self.avg_open_price,
                PositionSide::Short => (self.avg_open_price - avg_close_price) / self.avg_open_price,
            }
        } else {
            Decimal::ZERO
        }
    }
}

impl From<Fill> for Position {
    fn from(fill: Fill) -> Self {
        PositionBuilder::default()
            .instrument(fill.instrument)
            .side(fill.side)
            .avg_open_price(fill.price)
            .quantity(fill.quantity)
            .commission(fill.commission)
            .created_at(fill.created_at)
            .build()
            .expect("Failed to build position from fill")
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {} avg open: {} avg close: {} quantity: {} pnl: {} commission: {}",
            self.instrument,
            self.side,
            self.avg_open_price,
            self.avg_close_price.unwrap_or(Decimal::ZERO),
            self.quantity,
            self.realized_pnl,
            self.commission,
        )
    }
}
