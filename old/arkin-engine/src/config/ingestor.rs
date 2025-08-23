use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IngestorsConfig {
    pub ingestors: IngestorTypeConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IngestorTypeConfig {
    pub binance: Option<BinanceIngestorConfig>,
    pub tardis: Option<TardisIngestorConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinanceIngestorConfig {
    pub ws_url: String,
    pub connections_per_manager: usize,
    pub duplicate_lookback: usize,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub channels: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TardisIngestorConfig {
    pub http_url: String,
    pub api_secret: Option<String>,
    pub max_concurrent_requests: usize,
    pub venue: String,
    pub channel: String,
}
