use arkin_core::Event;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{ParseError, StreamConfig, VenueParser};

pub struct BybitSpotParser;

impl VenueParser for BybitSpotParser {
    fn parse(&self, msg: &str, config: &StreamConfig) -> Result<Event, ParseError> {
        // Implementation goes here
        unimplemented!()
    }
}

#[derive(Deserialize)]
struct BybitTrade {
    #[serde(rename = "i")]
    trade_id: String,
    #[serde(rename = "T")]
    timestamp: u64,
    #[serde(rename = "p")]
    price: Decimal,
    #[serde(rename = "v")]
    volume: Decimal,
    #[serde(rename = "S")]
    side: String,
    #[serde(rename = "s")]
    symbol: String,
}

#[derive(Deserialize)]
struct BybitTicker {
    symbol: String,
    #[serde(rename = "lastPrice")]
    last_price: Decimal,
    #[serde(rename = "bid1Price")]
    bid_price: Decimal,
    #[serde(rename = "bid1Size")]
    bid_qty: Decimal,
    #[serde(rename = "ask1Price")]
    ask_price: Decimal,
    #[serde(rename = "ask1Size")]
    ask_qty: Decimal,
    #[serde(rename = "highPrice24h")]
    high_24h: Decimal,
    #[serde(rename = "lowPrice24h")]
    low_24h: Decimal,
    #[serde(rename = "volume24h")]
    volume_24h: Decimal,
    #[serde(rename = "turnover24h")]
    turnover_24h: Decimal,
    #[serde(rename = "ts", default)]
    timestamp: u64,
}
