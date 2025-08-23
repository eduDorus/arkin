use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountingConfig {
    pub accounting: AccountingTypeConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountingTypeConfig {
    single_strategy: Option<SingleStrategyPortfolioConfig>,
    ledger: Option<LedgerPortfolioConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SingleStrategyPortfolioConfig {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LedgerPortfolioConfig {}
