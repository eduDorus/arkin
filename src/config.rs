use std::{collections::HashMap, env};

use config::{Config, Environment, File};
use serde::Deserialize;
use tracing::error;

#[derive(Debug, Deserialize, Clone)]
pub struct GlobalConfig {
    pub name: String,
    pub state: StateConfig,
    pub ingestors: IngestorFactoryConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StateConfig {
    pub market_state: MarketStateConfig,
    pub portfolio: PortfolioConfig,
    pub order_manager: OrderManagerConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MarketStateConfig {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PortfolioConfig {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OrderManagerConfig {
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IngestorFactoryConfig {
    pub binance: HashMap<String, BinanceIngestorConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceIngestorConfig {
    pub enabled: bool,
    pub ws_url: String,
    pub ws_channels: Vec<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub connections_per_manager: usize,
    pub duplicate_lookback: usize,
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
