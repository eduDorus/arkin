use std::env;

use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use tracing::error;

mod allocation;
mod clock;
mod db;
mod execution;
mod features;
mod ingestors;
mod server;
mod state;

pub use allocation::*;
pub use clock::*;
pub use db::*;
pub use execution::*;
pub use features::*;
pub use ingestors::*;
pub use server::*;
pub use state::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    pub server: ServerConfig,
    pub clock: ClockConfig,
    pub state: StateConfig,
    pub db: DBConfig,
    pub ingestors: Vec<IngestorConfig>,
    pub pipelines: Vec<PipelineConfig>,
    pub strategies: Vec<StrategyConfig>,
    pub allocation: AllocationConfig,
    pub execution: Vec<ExecutionConfig>,
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
