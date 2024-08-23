use config::{Config, Environment, File};
use serde::de::DeserializeOwned;
use std::env;
use tracing::error;

pub fn load<T: DeserializeOwned>() -> T {
    let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());

    let res = Config::builder()
        // .add_source(File::with_name("../configs/default"))
        .add_source(File::with_name(&format!("../configs/{}", run_mode)).required(false))
        .add_source(File::with_name(&format!("../configs/{}_secrets", run_mode)).required(false))
        .add_source(Environment::with_prefix("ARKIN"))
        .build();

    let loaded_config = match res {
        Ok(c) => c,
        Err(e) => {
            error!("Configuration error: {:?}", e);
            panic!("Failed to load configuration.");
        }
    };

    match loaded_config.try_deserialize::<T>() {
        Ok(c) => c,
        Err(e) => {
            error!("Configuration error: {:?}", e);
            panic!("Failed to load configuration.");
        }
    }
}
