use crate::{
    ingestors::IngestorID,
    models::{BookUpdate, BookUpdateSide, Event, Tick, Trade},
    utils::custom_serde,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use time::OffsetDateTime;

use super::parser::BinanceParser;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BinanceSwapsEvent {
    TradeStream(BinanceSwapsTrade),
    Trade(BinanceSwapsTradeData),
    AggTradeStream(BinanceSwapsAggTrade),
    AggTrade(BinanceSwapsAggTradeData),
    BookStream(BinanceSwapsBook),
    Book(BinanceSwapsBookData),
    TickStream(BinanceSwapsTick),
    Tick(BinanceSwapsTickData),
}

impl From<BinanceSwapsEvent> for Event {
    fn from(event: BinanceSwapsEvent) -> Self {
        match event {
            BinanceSwapsEvent::TradeStream(data) => Event::from(data.data),
            BinanceSwapsEvent::Trade(data) => Event::from(data),
            BinanceSwapsEvent::AggTradeStream(data) => Event::from(data.data),
            BinanceSwapsEvent::AggTrade(data) => Event::from(data),
            BinanceSwapsEvent::BookStream(data) => Event::from(data.data),
            BinanceSwapsEvent::Book(data) => Event::from(data),
            BinanceSwapsEvent::TickStream(data) => Event::from(data.data),
            BinanceSwapsEvent::Tick(data) => Event::from(data),
        }
    }
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
        Event::Trade(Trade::new(
            OffsetDateTime::now_utc(),
            data.event_time,
            instrument,
            data.trade_id,
            data.price.into(), // TODO: Fix this
            data.quantity.into(),
            IngestorID::Binance,
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
        Event::Trade(Trade::new(
            OffsetDateTime::now_utc(),
            data.event_time,
            instrument,
            data.agg_trade_id,
            data.price.into(), // TODO: Fix this
            data.quantity.into(),
            IngestorID::Binance,
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
        Event::Book(BookUpdate::new(
            data.event_time,
            instrument,
            data.bids
                .iter()
                .map(|b| BookUpdateSide::new(b.price.into(), b.quantity.into()))
                .collect(),
            data.asks
                .iter()
                .map(|a| BookUpdateSide::new(a.price.into(), a.quantity.into()))
                .collect(),
            IngestorID::Binance,
        ))
    }
}

// {
//     "e":"bookTicker",         // event type
//     "u":400900217,            // order book updateId
//     "E": 1568014460893,       // event time
//     "T": 1568014460891,       // transaction time
//     "s":"BNBUSDT",            // symbol
//     "b":"25.35190000",        // best bid price
//     "B":"31.21000000",        // best bid qty
//     "a":"25.36520000",        // best ask price
//     "A":"40.66000000"         // best ask qty
//   }

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
    pub update_id: u64,
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
        Event::Tick(Tick::new(
            data.event_time,
            instrument,
            data.update_id,
            data.bid_price.into(), // TODO: Fix this
            data.bid_quantity.into(),
            data.ask_price.into(), // TODO: Fix this
            data.ask_quantity.into(),
            IngestorID::Binance,
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
