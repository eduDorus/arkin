use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use time::OffsetDateTime;
use typed_builder::TypedBuilder;

use crate::Event;

use super::Asset;

#[derive(Debug, Clone, TypedBuilder, PartialEq)]
pub struct BalanceUpdate {
    pub event_time: OffsetDateTime,
    pub asset: Arc<Asset>,
    // pub quantity_change: Decimal,
    pub quantity: Decimal,
}

impl From<BalanceUpdate> for Event {
    fn from(holding: BalanceUpdate) -> Self {
        Event::BalanceUpdate(Arc::new(holding))
    }
}

impl From<Arc<BalanceUpdate>> for Event {
    fn from(holding: Arc<BalanceUpdate>) -> Self {
        Event::BalanceUpdate(holding)
    }
}

impl fmt::Display for BalanceUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "asset={} quantity={}", self.asset, self.quantity)
    }
}
