use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortfolioConfig {
    pub portfolio_manager: PortfolioManagerConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortfolioManagerConfig {
    pub initial_capital: Decimal,
    pub leverage: Decimal,
}
