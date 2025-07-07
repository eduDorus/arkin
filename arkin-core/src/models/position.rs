use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use sqlx::prelude::Type;
use strum::Display;
use time::OffsetDateTime;
use typed_builder::TypedBuilder;

use crate::{Price, Quantity};

use super::{Instrument, MarketSide};

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

#[derive(Debug, Clone, TypedBuilder)]
pub struct PositionUpdate {
    pub event_time: OffsetDateTime,
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
