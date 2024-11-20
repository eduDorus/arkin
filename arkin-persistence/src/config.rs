use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PersistenceConfig {
    pub database: DatabaseConfig,
    pub auto_commit_interval: u64,
    pub batch_size: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub min_connections: u32,
    pub max_connections: u32,
    pub idle_timeout: u64,
    pub acquire_timeout: u64,
    pub max_lifetime: u64,
}
