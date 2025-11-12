// Needs to parse this:
// live:
//   binance:
//     spot:
//       http:
//         url: "https://api.binance.com"
//         endpoints:
//           - channel: instruments
//             enabled: true
//             endpoint: "/api/v3/exchangeInfo"
//             polling_interval_secs: 300
//       ws:
//         url: "wss://stream.binance.com:9443/ws"
//         endpoints:
//           - channel: agg_trades
//             enabled: true
//             prefix: ""
//             suffix: "@aggTrade"
//           - channel: top_of_book
//             enabled: true
//             prefix: ""
//             suffix: "@bookTicker"
//     perpetual:
//       http:
//         url: "https://fapi.binance.com"
//         endpoints:
//           - channel: instruments
//             enabled: true
//             endpoint: "/fapi/v1/exchangeInfo"
//             polling_interval_secs: 300
//       ws:
//         url: "wss://fstream.binance.com/ws"
//         endpoints:
//           - channel: agg_trades
//             enabled: true
//             prefix: ""
//             suffix: "@aggTrade"
//           - channel: top_of_book
//             enabled: true
//             prefix: ""
//             suffix: "@bookTicker"
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct IngestorConfig {
    pub live: LiveConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LiveConfig {
    pub binance: BinanceConfig,
    // Add other exchanges here as needed
}

#[derive(Debug, Deserialize, Clone)]
pub struct BinanceConfig {
    pub spot: VenueConfig,
    pub perpetual: VenueConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct VenueConfig {
    pub http: HttpConfig,
    pub ws: WsConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpConfig {
    pub url: String,
    pub endpoints: Vec<HttpEndpointConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct HttpEndpointConfig {
    pub channel: String,
    pub enabled: bool,
    pub method: String,
    pub endpoint: String,
    pub polling_interval_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WsConfig {
    pub url: String,
    pub endpoints: Vec<WsEndpointConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct WsEndpointConfig {
    pub channel: String,
    pub enabled: bool,
    pub prefix: String,
    pub suffix: String,
}
