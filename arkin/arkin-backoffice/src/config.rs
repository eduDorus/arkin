use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortfolioManagerConfig {
    pub initial_capital: Decimal,
    pub leverage: Decimal,
    pub initial_margin: Decimal,
    pub maintenance_margin: Decimal,
}
