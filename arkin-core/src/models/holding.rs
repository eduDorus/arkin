use std::fmt;

use derive_builder::Builder;
use rust_decimal::Decimal;

use crate::{types::AssetId, Event, UpdateEvent, UpdateEventType};

#[derive(Debug, Clone, Builder, PartialEq)]
pub struct Holding {
    pub asset: AssetId,
    pub quantity: Decimal,
}

impl Event for Holding {
    fn event_type() -> UpdateEventType {
        UpdateEventType::Balance
    }
}

impl From<Holding> for UpdateEvent {
    fn from(holding: Holding) -> Self {
        UpdateEvent::Balance(holding)
    }
}

impl fmt::Display for Holding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Asset: {} balance: {}", self.asset, self.quantity)
    }
}
