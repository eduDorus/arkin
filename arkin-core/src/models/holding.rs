use std::fmt;

use derive_builder::Builder;
use rust_decimal::Decimal;

use crate::types::AssetId;

#[derive(Debug, Clone, Builder)]
pub struct Holding {
    pub asset: AssetId,
    pub quantity: Decimal,
}

impl fmt::Display for Holding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Asset: {} balance: {}", self.asset, self.quantity)
    }
}
