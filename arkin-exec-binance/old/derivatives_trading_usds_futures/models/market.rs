#![allow(unused)]
use std::fmt;

use rust_decimal::Decimal;
use serde::Deserialize;
use time::UtcDateTime;

use arkin_core::prelude::*;

// Re-export core models for convenience
// pub use arkin_core::models::exchange::*;

/// Live market data stream event from Binance
#[derive(Debug, Deserialize)]
pub struct BinanceUSDMMarketStreamEvent {
    pub stream: String,
    pub data: BinanceUSDMMarketEvent, // Tagged enum based on "e" field
}

/// Tagged enum for live Binance market events
#[derive(Debug, Deserialize)]
#[serde(tag = "e")]
pub enum BinanceUSDMMarketEvent {
    #[serde(rename = "aggTrade")]
    AggTrade(BinanceSwapsAggTradeData),
    #[serde(rename = "bookTicker")]
    Tick(BinanceSwapsTickData),
    #[serde(rename = "depthUpdate")]
    Book(BinanceSwapsBookData),
    #[serde(rename = "trade")]
    Trade(BinanceSwapsTradeData),
}

// https://api.tardis.dev/v1/exchanges
// {
//     "id": "binance-futures",
//     "name": "Binance USDT Futures",
//     "enabled": true,
//     "supportsDatasets": true,
//     "availableSince": "2019-11-17T00:00:00.000Z",
//     "availableChannels": [
//         "trade",
//         "aggTrade",
//         "ticker",
//         "depth",
//         "depthSnapshot",
//         "markPrice",
//         "bookTicker",
//         "forceOrder",
//         "openInterest",
//         "recentTrades",
//         "compositeIndex",
//         "topLongShortAccountRatio",
//         "topLongShortPositionRatio",
//         "globalLongShortAccountRatio",
//         "takerlongshortRatio"
//     ]
// },

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BinanceSwapsListenKeyResponse {
    pub listen_key: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binance_futures_trade() {
        let json_data = r#"{"stream":"btcusdt@trade","data":{"e":"trade","E":1676160600276,"T":1676160600269,"s":"BTCUSDT","t":3280342045,"p":"21845.10","q":"0.001","X":"MARKET","m":false}}"#;
        let _ = serde_json::from_str::<BinanceUSDMMarketStreamEvent>(json_data).unwrap();
    }

    #[test]
    fn test_binance_futures_agg_trade() {
        let json_data = r#"{"stream":"gasusdt@aggTrade","data":{"e":"aggTrade","E":1698796800043,"a":3863267,"s":"GASUSDT","p":"6.279000","q":"141.2","f":15146241,"l":15146244,"T":1698796799890,"m":false}}"#;
        let _ = serde_json::from_str::<BinanceUSDMMarketStreamEvent>(json_data).unwrap();
    }

    #[test]
    fn test_binance_futures_book() {
        let json_data = r#"{"stream":"ethusdt@depth@0ms","data":{"e":"depthUpdate","E":1676024912112,"T":1676024912105,"s":"ETHUSDT","U":2487393816175,"u":2487393817473,"pu":2487393816141,"b":[["1543.11","18.655"],["1543.14","19.449"],["1543.17","0.004"],["1543.37","27.385"],["1543.82","87.566"],["1543.89","10.302"],["1543.93","0.000"],["1544.13","4.546"]],"a":[["1544.11","0.000"],["1544.13","0.000"],["1544.22","1.105"],["1544.44","42.495"],["1544.52","13.691"]]}}"#;
        let _ = serde_json::from_str::<BinanceUSDMMarketStreamEvent>(json_data).unwrap();
    }

    #[test]
    fn test_binance_futures_ticker() {
        let json_data = r#"{"stream":"btcusdt@bookTicker","data":{"e":"bookTicker","u":2487455691211,"s":"BTCUSDT","b":"21840.40","B":"21.292","a":"21840.50","A":"11.169","T":1676026461537,"E":1676026461542}}"#;
        let _ = serde_json::from_str::<BinanceUSDMMarketStreamEvent>(json_data).unwrap();
    }

    #[test]
    fn test_binance_futures_ticker_data() {
        let json_data = r#"{"e":"bookTicker","u":5790517108882,"s":"BTCUSDT","b":"91252.80","B":"2.913","a":"91252.90","A":"4.781","T":1731728671784,"E":1731728671784}"#;
        let _ = serde_json::from_str::<BinanceUSDMMarketEvent>(json_data).unwrap();
    }
}
