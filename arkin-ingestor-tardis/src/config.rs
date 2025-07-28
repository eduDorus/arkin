use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct TardisConfig {
    pub tardis: TardisIngestorConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TardisIngestorConfig {
    pub http_url: String,
    pub api_secret: Option<String>,
    pub max_concurrent_requests: usize,
    // pub venue: String,
    // pub channel: String,
}
