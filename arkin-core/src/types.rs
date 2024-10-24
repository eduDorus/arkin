use std::sync::Arc;

use rust_decimal::prelude::*;
use time::OffsetDateTime;

pub type StrategyId = String;
pub type FeatureId = Arc<String>;

// Common types
pub type Price = Decimal;
pub type Quantity = Decimal;
pub type Notional = Decimal;
pub type Weight = Decimal;
pub type Maturity = OffsetDateTime;
pub type Commission = Decimal;
