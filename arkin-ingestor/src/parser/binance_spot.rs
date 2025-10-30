use arkin_core::Event;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{ParseError, VenueParser};

pub struct BinanceSpotParser;

impl VenueParser for BinanceSpotParser {
    fn parse(&self, msg: &str, config: &StreamConfig) -> Result<Event, ParseError> {
        // Implementation goes here
        unimplemented!()
    }
}

#[derive(Deserialize)]
struct BinanceAggregateTrade {
    #[serde(rename = "a")]
    trade_id: u64,
    #[serde(rename = "p")]
    price: Decimal,
    #[serde(rename = "q")]
    quantity: Decimal,
    #[serde(rename = "T")]
    trade_time: u64,
    #[serde(rename = "m")]
    is_buyer_maker: bool,
}

#[derive(Deserialize)]
struct BinanceTicker24h {
    #[serde(rename = "E")]
    event_time: u64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "c")]
    last_price: Decimal,
    #[serde(rename = "b")]
    bid_price: Decimal,
    #[serde(rename = "B")]
    bid_qty: Decimal,
    #[serde(rename = "a")]
    ask_price: Decimal,
    #[serde(rename = "A")]
    ask_qty: Decimal,
    #[serde(rename = "h")]
    high_price: Decimal,
    #[serde(rename = "l")]
    low_price: Decimal,
    #[serde(rename = "v")]
    volume: Decimal,
    #[serde(rename = "q")]
    volume_quote: Decimal,
}

#[derive(Deserialize)]
struct BinanceMarkPrice {
    #[serde(rename = "E")]
    event_time: u64,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "p")]
    mark_price: Decimal,
}
