use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IngestorConfig {
    pub ingestor_service: IngestorServiceConfig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IngestorServiceConfig {
    pub ingestors: Vec<IngestorModuleConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IngestorModuleConfig {
    #[serde(rename = "binance")]
    Binance(BinanceIngestorConfig),
    #[serde(rename = "tardis")]
    Tardis(TardisIngestorConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinanceIngestorConfig {
    pub ws_url: String,
    pub ws_channels: Vec<String>,
    pub api_key: Option<String>,
    pub api_secret: Option<String>,
    pub connections_per_manager: usize,
    pub duplicate_lookback: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TardisIngestorConfig {
    pub api_secret: Option<String>,
    pub base_url: String,
    pub max_concurrent_requests: usize,
}
