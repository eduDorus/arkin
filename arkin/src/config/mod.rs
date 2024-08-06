use config::{Config, Environment, File};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::env;
use tracing::error;

mod allocation;
mod clock;
mod db;
mod execution;
mod features;
mod ingestors;
mod server;
mod state;
mod strategy;

pub use allocation::*;
pub use clock::*;
pub use db::*;
pub use execution::*;
pub use features::*;
pub use ingestors::*;
pub use server::*;
pub use state::*;
pub use strategy::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    pub server: ServerConfig,
    pub clock: ClockConfig,
    pub state: StateConfig,
    pub db: DatabaseConfig,
    pub ingestors: Vec<IngestorConfig>,
    pub pipeline: PipelineConfig,
    pub strategy_manager: StrategyManagerConfig,
    pub allocation_manager: AllocationManagerConfig,
    pub execution: Vec<ExecutionConfig>,
    pub simulation: SimulationConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SimulationConfig {
    pub latency: u64, // in milliseconds
    pub commission_maker: Decimal,
    pub commission_taker: Decimal,
    pub max_orders_per_minute: u64,
    pub max_order_size_notional: Decimal,
    pub min_order_size_notional: Decimal,
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
