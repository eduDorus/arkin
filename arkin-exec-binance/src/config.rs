use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceExecutionServiceConfig {
    pub binance_execution: BinanceExecutionConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceExecutionConfig {
    pub spot: Option<BinanceSpotExecutionConfig>,
    pub margin: Option<BinanceMarginExecutionConfig>,
    pub usdm: Option<BinanceUsdmExecutionConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceSpotExecutionConfig {
    pub enabled: bool,
    pub api_key: String,
    pub api_secret: String,
    pub base_url: Url,
    pub testnet: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceMarginExecutionConfig {
    pub enabled: bool,
    pub api_key: String,
    pub api_secret: String,
    pub base_url: Url,
    pub testnet: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceUsdmExecutionConfig {
    pub enabled: bool,
    pub api_key: String,
    pub api_secret: String,
    pub base_url: Url,
    pub testnet: bool,
}
