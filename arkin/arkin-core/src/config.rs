use config::{Config, Environment, File};
use serde::de::DeserializeOwned;
use std::env;
use tracing::{debug, error};

pub fn load<T: DeserializeOwned>() -> T {
    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());

    let config = Config::builder()
        // .add_source(File::with_name("../configs/default"))
        .add_source(File::with_name(&format!("../configs/{}", run_mode)).required(false))
        .add_source(File::with_name(&format!("../configs/{}_secrets", run_mode)).required(false))
        .add_source(Environment::with_prefix("ARKIN"))
        .build()
        .expect("Failed to build configuration");

    debug!("Loading configuration for: {:?}", config);

    match config.try_deserialize::<T>() {
        Ok(c) => c,
        Err(e) => {
            error!("Configuration error: {:?}", e);
            panic!("Failed to load configuration.");
        }
    }
}
