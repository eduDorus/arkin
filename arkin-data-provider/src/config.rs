use arkin_core::Channel;
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize, Clone)]
pub struct DataProviderConfig {
    pub data_providers: Vec<ProviderConfig>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderConfig {
    BinanceSpot(BinanceSpotConfig),
    BinanceUsdm(BinanceUsdmConfig),
    BinanceUsdmUser(BinanceUsdmUserConfig),
    BinanceSpotUser(BinanceSpotUserConfig),
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpEndpointConfig {
    pub channel: Channel,
    pub enabled: bool,
    pub method: String,
    pub endpoint: String,
    pub polling_interval_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WsEndpointConfig {
    pub channel: Channel,
    pub enabled: bool,
    pub prefix: String,
    pub suffix: String,
    pub symbols: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceUsdmConfig {
    pub http_url: Url,
    pub ws_url: Url,
    #[serde(default)]
    pub http_endpoints: Vec<HttpEndpointConfig>,
    #[serde(default)]
    pub ws_endpoints: Vec<WsEndpointConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceUsdmUserConfig {
    pub api_key: String,
    pub api_secret: String,
    pub http_url: Url,
    pub ws_url: Url,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceSpotConfig {
    pub http_url: Url,
    pub ws_url: Url,
    #[serde(default)]
    pub http_endpoints: Vec<HttpEndpointConfig>,
    #[serde(default)]
    pub ws_endpoints: Vec<WsEndpointConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceSpotUserConfig {
    pub api_key: String,
    pub api_secret: String,
    pub http_url: Url,
    pub ws_url: Url,
    pub enabled: bool,
}
