use std::fmt;

use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{BalanceUpdate, PositionUpdate};

#[derive(Debug, Clone, TypedBuilder)]
pub struct AccountUpdate {
    pub id: Uuid,
    pub event_time: UtcDateTime,
    pub balances: Vec<BalanceUpdate>,
    pub positions: Vec<PositionUpdate>,
    pub reason: String, // "m" from stream, e.g., "ORDER"
}

impl PartialEq for AccountUpdate {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AccountUpdate {}

impl fmt::Display for AccountUpdate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "balance_updates={} position_updates={}",
            self.balances.len(),
            self.positions.len()
        )
    }
}
