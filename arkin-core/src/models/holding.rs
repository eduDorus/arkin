use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{Event, EventType, EventTypeOf};

use super::Asset;

#[derive(Debug, Clone, TypedBuilder, PartialEq)]
pub struct Holding {
    pub id: Uuid,
    pub asset: Arc<Asset>,
    pub balance: Decimal,
}

impl EventTypeOf for Holding {
    fn event_type() -> EventType {
        EventType::BalanceUpdate
    }
}

impl From<Arc<Holding>> for Event {
    fn from(holding: Arc<Holding>) -> Self {
        Event::BalanceUpdate(holding)
    }
}

impl fmt::Display for Holding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "asset={} balance={}", self.asset, self.balance)
    }
}
