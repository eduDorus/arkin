use std::fmt;

use derive_builder::Builder;
use rust_decimal::Decimal;

use crate::{types::AssetId, Event, EventType, EventTypeOf};

#[derive(Debug, Clone, Builder, PartialEq)]
pub struct Holding {
    pub asset: AssetId,
    pub balance: Decimal,
}

impl EventTypeOf for Holding {
    fn event_type() -> EventType {
        EventType::BalanceUpdate
    }
}

impl From<Holding> for Event {
    fn from(holding: Holding) -> Self {
        Event::BalanceUpdate(holding)
    }
}

impl fmt::Display for Holding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "asset={} balance={}", self.asset, self.balance)
    }
}
