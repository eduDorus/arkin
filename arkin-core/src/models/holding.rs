use std::{fmt, sync::Arc};

use derive_builder::Builder;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{Event, EventType, EventTypeOf};

use super::{Asset, Instance};

#[derive(Debug, Clone, Builder, PartialEq)]
pub struct Holding {
    pub id: Uuid,
    pub instance: Arc<Instance>,
    pub asset: Arc<Asset>,
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
