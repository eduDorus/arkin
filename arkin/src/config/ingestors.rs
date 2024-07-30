use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IngestorConfig {
    #[serde(rename = "backtest")]
    Backtest(BacktestIngestorConfig),
    #[serde(rename = "binance")]
    Binance(BinanceIngestorConfig),
    #[serde(rename = "tardis")]
    Tardis(TardisIngestorConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BacktestIngestorConfig {
    pub market_data: bool,
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
