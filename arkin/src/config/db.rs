use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DBConfig {
    pub url: String,
    pub user: String,
    pub password: String,
    pub database: String,
}
