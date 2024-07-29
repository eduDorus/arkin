use std::env;

use config::{Config, Environment, File};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::features::FeatureID;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    pub name: String,
    pub state: StateConfig,
    pub ingestors: Vec<IngestorConfig>,
    pub pipelines: Vec<PipelineConfig>,
    pub strategies: Vec<StrategyConfig>,
    pub allocation: AllocationConfig,
    pub execution: Vec<ExecutionConfig>,
    pub tardis: TardisConfig,
    pub clickhouse: ClickhouseConfig,
}

// STATE
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateConfig {
    pub market_state: MarketStateConfig,
    pub portfolio: PortfolioConfig,
    pub order_manager: OrderManagerConfig,
    pub time_component: TimeComponentConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketStateConfig {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortfolioConfig {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderManagerConfig {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimeComponentConfig {
    pub tick_frequency: u64,
}

// INGESTORS
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IngestorConfig {
    #[serde(rename = "backtest")]
    Backtest(BacktestIngestorConfig),
    #[serde(rename = "binance")]
    Binance(BinanceIngestorConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BacktestIngestorConfig {
    pub market_data: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinanceIngestorConfig {
    pub ws_url: String,
    pub ws_channels: Vec<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub connections_per_manager: usize,
    pub duplicate_lookback: usize,
}

// PIPELINES
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineConfig {
    pub name: String,
    pub frequency: u64,
    pub features: Vec<FeatureConfig>,
}

// FEATURES
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FeatureConfig {
    #[serde(rename = "volume")]
    Volume(VolumeFeatureConfig),
    #[serde(rename = "vwap")]
    VWAP(VWAPFeatureConfig),
    #[serde(rename = "sma")]
    SMA(SMAFeatureConfig),
    #[serde(rename = "ema")]
    EMA(EMAFeatureConfig),
    #[serde(rename = "spread")]
    Spread(SpreadFeatureConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolumeFeatureConfig {
    pub id: FeatureID,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VWAPFeatureConfig {
    pub id: FeatureID,
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SMAFeatureConfig {
    pub id: FeatureID,
    pub source: FeatureID,
    pub period: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EMAFeatureConfig {
    pub id: FeatureID,
    pub source: FeatureID,
    pub period: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpreadFeatureConfig {
    pub id: FeatureID,
    pub front_component: FeatureID,
    pub back_component: FeatureID,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StrategyConfig {
    #[serde(rename = "crossover")]
    Crossover(CrossoverConfig),
    #[serde(rename = "spreader")]
    Spreader(SpreaderConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrossoverConfig {
    pub id: String,
    pub fast: String,
    pub slow: String,
    pub min_spread: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpreaderConfig {
    pub id: String,
    pub front_leg: String,
    pub back_leg: String,
    pub min_spread: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AllocationConfig {
    #[serde(rename = "limited")]
    Limited(LimitedAllocationConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LimitedAllocationConfig {
    pub max_allocation: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExecutionConfig {
    #[serde(rename = "backtest")]
    Backtest(BacktestExecutionConfig),
    #[serde(rename = "binance")]
    Binance(BinanceExecutionConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinanceExecutionConfig {
    pub max_orders_per_minute: u64,
    pub max_order_size_notional: Decimal,
    pub min_order_size_notional: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BacktestExecutionConfig {
    pub max_orders_per_minute: u64,
    pub max_order_size_notional: Decimal,
    pub min_order_size_notional: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TardisConfig {
    pub api_secret: Option<String>,
    pub base_url: String,
    pub max_concurrent_requests: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClickhouseConfig {
    pub url: String,
    pub user: String,
    pub password: String,
    pub database: String,
}

pub fn load() -> GlobalConfig {
    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());

    let res = Config::builder()
        .add_source(File::with_name("configs/default"))
        .add_source(File::with_name(&format!("configs/{}", run_mode)).required(false))
        .add_source(File::with_name(&format!("configs/{}_secrets", run_mode)).required(false))
        .add_source(Environment::with_prefix("AURELION"))
        .build();

    let loaded_config = match res {
        Ok(c) => c,
        Err(e) => {
            error!("Configuration error: {:?}", e);
            panic!("Failed to load configuration.");
        }
    };

    match loaded_config.try_deserialize::<GlobalConfig>() {
        Ok(c) => c,
        Err(e) => {
            error!("Configuration error: {:?}", e);
            panic!("Failed to load configuration.");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::logging;

    use super::*;

    #[test]
    fn test_parse_config() {
        logging::init_test_tracing();
        load();
    }
}
