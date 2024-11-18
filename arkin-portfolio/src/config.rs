use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortfolioConfig {
    pub portfolio: PortfolioType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PortfolioType {
    #[serde(rename = "single_strategy")]
    SingleStrategy(SingleStrategyPortfolioConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SingleStrategyPortfolioConfig {
    pub initial_capital: Decimal,
    pub leverage: Decimal,
}
