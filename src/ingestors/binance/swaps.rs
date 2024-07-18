use crate::{
    models::{BookUpdate, BookUpdateSide, Event, Price, Quantity, Tick, Trade},
    utils::custom_serde,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use time::OffsetDateTime;

use super::parser::BinanceParser;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BinanceSwapsEvent {
    Trade(BinanceSwapsTradeData),
    AggTrade(BinanceSwapsAggTradeData),
    Book(BinanceSwapsBookData),
    Tick(BinanceSwapsTickData),
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
#[allow(unused)]
pub struct BinanceSwapsTrade {
    pub stream: String,
    pub data: BinanceSwapsTradeData,
}

#[derive(Debug, Deserialize)]
pub struct BinanceSwapsTradeData {
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    pub event_time: OffsetDateTime,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: OffsetDateTime,
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "s")]
    pub instrument: String,
    #[serde(rename = "t")]
    pub trade_id: u64,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "q")]
    pub quantity: Decimal,
    #[serde(rename = "X")]
    pub trade_type: String,
    #[serde(rename = "m")]
    pub maker: bool, // The true = sell, false = buy
}

impl From<BinanceSwapsTradeData> for Event {
    fn from(data: BinanceSwapsTradeData) -> Self {
        let instrument = BinanceParser::parse_instrument(&data.instrument);
        Event::TradeUpdate(Trade::new(
            instrument,
            data.event_time,
            Price::new(data.price).unwrap(), // TODO: Fix this
            Quantity::new(data.quantity),
        ))
    }
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct BinanceSwapsAggTrade {
    pub stream: String,
    pub data: BinanceSwapsAggTradeData,
}

#[derive(Debug, Deserialize)]
pub struct BinanceSwapsAggTradeData {
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    pub event_time: OffsetDateTime,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: OffsetDateTime,
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "s")]
    pub instrument: String,
    #[serde(rename = "a")]
    pub agg_trade_id: u64,
    #[serde(rename = "f")]
    pub first_trade_id: u64,
    #[serde(rename = "l")]
    pub last_trade_id: u64,
    #[serde(rename = "p")]
    pub price: Decimal,
    #[serde(rename = "q")]
    pub quantity: Decimal,
    #[serde(rename = "m")]
    pub maker: bool, // The true = sell, false = buy
}

impl From<BinanceSwapsAggTradeData> for Event {
    fn from(data: BinanceSwapsAggTradeData) -> Self {
        let instrument = BinanceParser::parse_instrument(&data.instrument);
        Event::AggTradeUpdate(Trade::new(
            instrument,
            data.event_time,
            Price::new(data.price).unwrap(), // TODO: Fix this
            Quantity::new(data.quantity),
        ))
    }
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct BinanceSwapsBook {
    pub stream: String,
    pub data: BinanceSwapsBookData,
}

#[derive(Debug, Deserialize)]
pub struct BinanceSwapsBookData {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    pub event_time: OffsetDateTime,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: OffsetDateTime,
    #[serde(rename = "s")]
    pub instrument: String,
    #[serde(rename = "U")]
    pub first_update_id: u64,
    #[serde(rename = "u")]
    pub final_update_id: u64,
    #[serde(rename = "pu")]
    pub last_final_update_id: u64,
    #[serde(rename = "b")]
    pub bids: Vec<BinanceSwapsBookUpdate>,
    #[serde(rename = "a")]
    pub asks: Vec<BinanceSwapsBookUpdate>,
}

#[derive(Debug, Deserialize)]
pub struct BinanceSwapsBookUpdate {
    pub price: Decimal,
    pub quantity: Decimal,
}

impl From<BinanceSwapsBookData> for Event {
    fn from(data: BinanceSwapsBookData) -> Self {
        let instrument = BinanceParser::parse_instrument(&data.instrument);
        Event::BookUpdate(BookUpdate {
            instrument,
            event_time: data.event_time,
            bids: data
                .bids
                .iter()
                .map(|b| BookUpdateSide::new(Price::new(b.price).unwrap(), Quantity::new(b.quantity)))
                .collect(),
            asks: data
                .asks
                .iter()
                .map(|a| BookUpdateSide::new(Price::new(a.price).unwrap(), Quantity::new(a.quantity)))
                .collect(),
        })
    }
}

// {
//     "e": "24hrTicker",  // Event type
//     "E": 123456789,     // Event time
//     "s": "BTCUSDT",     // Symbol
//     "p": "0.0015",      // Price change
//     "P": "250.00",      // Price change percent
//     "w": "0.0018",      // Weighted average price
//     "c": "0.0025",      // Last price
//     "Q": "10",          // Last quantity
//     "o": "0.0010",      // Open price
//     "h": "0.0025",      // High price
//     "l": "0.0010",      // Low price
//     "v": "10000",       // Total traded base asset volume
//     "q": "18",          // Total traded quote asset volume
//     "O": 0,             // Statistics open time
//     "C": 86400000,      // Statistics close time
//     "F": 0,             // First trade ID
//     "L": 18150,         // Last trade Id
//     "n": 18151          // Total number of trades
// }

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct BinanceSwapsTick {
    pub stream: String,
    pub data: BinanceSwapsTickData,
}

#[derive(Debug, Deserialize)]
pub struct BinanceSwapsTickData {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    pub event_time: OffsetDateTime,
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: OffsetDateTime,
    #[serde(rename = "u")]
    pub update_id: i64,
    #[serde(rename = "s")]
    pub instrument: String,
    #[serde(rename = "b")]
    pub bid_price: Decimal,
    #[serde(rename = "B")]
    pub bid_quantity: Decimal,
    #[serde(rename = "a")]
    pub ask_price: Decimal,
    #[serde(rename = "A")]
    pub ask_quantity: Decimal,
}

impl From<BinanceSwapsTickData> for Event {
    fn from(data: BinanceSwapsTickData) -> Self {
        let instrument = BinanceParser::parse_instrument(&data.instrument);
        Event::TickUpdate(Tick::new(
            instrument,
            data.event_time,
            Price::new(data.bid_price).unwrap(), // TODO: Fix this
            Quantity::new(data.bid_quantity),
            Price::new(data.ask_price).unwrap(), // TODO: Fix this
            Quantity::new(data.ask_quantity),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binance_futures_trade() {
        let json_data = r#"{"stream":"btcusdt@trade","data":{"e":"trade","E":1676160600276,"T":1676160600269,"s":"BTCUSDT","t":3280342045,"p":"21845.10","q":"0.001","X":"MARKET","m":false}}"#;
        let _ = serde_json::from_str::<BinanceSwapsTrade>(json_data).unwrap();
    }

    #[test]

    fn test_binance_futures_agg_trade() {
        let json_data = r#"{"stream":"gasusdt@aggTrade","data":{"e":"aggTrade","E":1698796800043,"a":3863267,"s":"GASUSDT","p":"6.279000","q":"141.2","f":15146241,"l":15146244,"T":1698796799890,"m":false}}"#;
        let _ = serde_json::from_str::<BinanceSwapsAggTrade>(json_data).unwrap();
    }

    #[test]
    fn test_binance_futures_book() {
        let json_data = r#"{"stream":"ethusdt@depth@0ms","data":{"e":"depthUpdate","E":1676024912112,"T":1676024912105,"s":"ETHUSDT","U":2487393816175,"u":2487393817473,"pu":2487393816141,"b":[["1543.11","18.655"],["1543.14","19.449"],["1543.17","0.004"],["1543.37","27.385"],["1543.82","87.566"],["1543.89","10.302"],["1543.93","0.000"],["1544.13","4.546"]],"a":[["1544.11","0.000"],["1544.13","0.000"],["1544.22","1.105"],["1544.44","42.495"],["1544.52","13.691"]]}}"#;
        let _ = serde_json::from_str::<BinanceSwapsBook>(json_data).unwrap();
    }

    #[test]
    fn test_binance_futures_ticker() {
        let json_data = r#"{"stream":"btcusdt@bookTicker","data":{"e":"bookTicker","u":2487455691211,"s":"BTCUSDT","b":"21840.40","B":"21.292","a":"21840.50","A":"11.169","T":1676026461537,"E":1676026461542}}"#;
        let _ = serde_json::from_str::<BinanceSwapsTick>(json_data).unwrap();
    }

    #[test]
    #[ignore]
    fn test_binance_futures_ticker_2() {
        let json_data = r#"{"e":"24hrTicker","E":1720514702587,"s":"BTCUSDT","p":"697.00","P":"1.220","w":"56741.24","c":"57820.20","Q":"0.002","o":"57123.20","h":"58200.00","l":"54890.00","v":"388968.569","q":"22070559902.21","O":1720428300000,"C":1720514702585,"F":5147088255,"L":5151564448,"n":4476166}"#;
        let _ = serde_json::from_str::<BinanceSwapsTickData>(json_data).unwrap();
    }
}
