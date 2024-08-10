use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateManagerConfig {
    pub portfolio: PortfolioStateConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortfolioStateConfig {
    pub initial_capital: Decimal,
}
