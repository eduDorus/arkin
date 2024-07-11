use std::{collections::HashMap, env};

use config::{Config, Environment, File};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    pub name: String,
    pub state: StateConfig,
    pub ingestors: Vec<IngestorConfig>,
    pub features: Vec<FeatureConfig>,
    pub traders: HashMap<String, TraderConfig>,
}

// STATE
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateConfig {
    pub market_state: MarketStateConfig,
    pub portfolio: PortfolioConfig,
    pub order_manager: OrderManagerConfig,
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

// INGESTORS
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IngestorConfig {
    #[serde(rename = "binance")]
    Binance(BinanceIngestorConfig),
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

// FEATURES
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum FeatureConfig {
    #[serde(rename = "vwap")]
    VWAP(VWAPConfig),
    #[serde(rename = "sma")]
    SMA(SMAConfig),
    #[serde(rename = "ema")]
    EMA(EMAConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VWAPConfig {
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SMAConfig {
    pub window: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EMAConfig {
    pub window: u64,
}

// TRADER
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TraderConfig {
    pub strategy: StrategyConfig,
    pub allocation: AllocationConfig,
    pub execution: ExecutionConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StrategyConfig {
    #[serde(rename = "wide")]
    WideQuoter(WideQuoterConfig),
    #[serde(rename = "spreader")]
    Spreader(SpreaderConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WideQuoterConfig {
    pub spread_in_pct: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpreaderConfig {
    pub front_leg: String,
    pub back_leg: String,
    pub min_spread: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AllocationConfig {
    #[serde(rename = "fixed")]
    Fixed(FixedAllocationConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FixedAllocationConfig {
    pub max_allocation: Decimal,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExecutionConfig {
    #[serde(rename = "forward")]
    Forward(ForwardExecutionConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ForwardExecutionConfig {
    pub max_order_size: Decimal,
    pub min_order_size: Decimal,
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
