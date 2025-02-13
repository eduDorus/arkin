use config::{Config, Environment, File};
use serde::de::DeserializeOwned;
use std::env;
use tracing::{debug, error};

pub fn load<T: DeserializeOwned>() -> T {
    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());
    let config_dir = env::var("CONFIG_DIR").unwrap_or_else(|_| ".".into());

    let config = Config::builder()
        .add_source(File::with_name(&format!("{}/{}", config_dir, run_mode)).required(false))
        .add_source(File::with_name(&format!("{}/{}_persistence", config_dir, run_mode)).required(false))
        .add_source(File::with_name(&format!("{}/{}_ingestors", config_dir, run_mode)).required(false))
        .add_source(File::with_name(&format!("{}/{}_insights", config_dir, run_mode)).required(false))
        .add_source(File::with_name(&format!("{}/{}_secrets", config_dir, run_mode)).required(false))
        .add_source(Environment::with_prefix("ARKIN"))
        .build()
        .expect("Failed to build configuration");

    debug!("Loading configuration from: {}", config_dir);

    match config.try_deserialize::<T>() {
        Ok(c) => c,
        Err(e) => {
            error!("Configuration error: {:?}", e);
            panic!("Failed to load configuration.");
        }
    }
}
