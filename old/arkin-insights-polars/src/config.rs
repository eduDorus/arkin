use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolarsPipelineConfig {
    pub name: String,
    pub description: String,
    pub windows: Vec<String>,     // e.g. ["5m", "15m", "60m"]
    pub buffer_size_minutes: i64, // How much history to keep in memory
}

impl Default for PolarsPipelineConfig {
    fn default() -> Self {
        Self {
            name: "default_pipeline".to_string(),
            description: "Default Polars Pipeline".to_string(),
            windows: vec!["5m".to_string(), "15m".to_string(), "60m".to_string()],
            buffer_size_minutes: 1440, // 24 hours
        }
    }
}
