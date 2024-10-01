use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketConfig {
    pub market_manager: MarketManagerConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketManagerConfig {
    pub lookback_min: u64,
}
