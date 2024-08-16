use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortfolioConfig {
    pub initial_capital: Decimal,
    pub leverage: Decimal,
}
