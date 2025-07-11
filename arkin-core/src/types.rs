use std::sync::Arc;

use rust_decimal::prelude::*;
use time::UtcDateTime;

pub type FeatureId = Arc<String>;

// Common types
pub type Price = Decimal;
pub type Quantity = Decimal;
pub type MarketValue = Decimal;
pub type Notional = Decimal;
pub type Weight = Decimal;
pub type Maturity = UtcDateTime;
pub type Commission = Decimal;
