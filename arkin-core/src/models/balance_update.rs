use std::{fmt, sync::Arc};

use rust_decimal::Decimal;
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{AccountType, Venue};

use super::Asset;

#[derive(Debug, Clone, TypedBuilder)]
pub struct BalanceUpdate {
    #[builder(default)]
    pub id: Uuid,
    pub event_time: UtcDateTime,
    pub venue: Arc<Venue>,
    pub account_type: AccountType,
    pub asset: Arc<Asset>,
    pub quantity_change: Decimal,
    pub quantity: Decimal,
}

impl PartialEq for BalanceUpdate {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for BalanceUpdate {}

impl fmt::Display for BalanceUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "asset={} quantity={}", self.asset, self.quantity)
    }
}
