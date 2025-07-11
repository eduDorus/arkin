use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use time::UtcDateTime;
use typed_builder::TypedBuilder;

use super::Asset;

#[derive(Debug, Clone, TypedBuilder, PartialEq)]
pub struct BalanceUpdate {
    pub event_time: UtcDateTime,
    pub asset: Arc<Asset>,
    // pub quantity_change: Decimal,
    pub quantity: Decimal,
}

impl fmt::Display for BalanceUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "asset={} quantity={}", self.asset, self.quantity)
    }
}
