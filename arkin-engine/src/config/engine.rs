use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineConfig {
    pub engine: EngineTypeConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineTypeConfig {
    pub default: Option<EngineDefaultConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EngineDefaultConfig {
    pub pubsub_capacity: usize,
}
