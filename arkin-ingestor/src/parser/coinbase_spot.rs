use arkin_core::Event;
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::{ParseError, StreamConfig, VenueParser};

pub struct CoinbaseSpotParser;

impl VenueParser for CoinbaseSpotParser {
    fn parse(&self, msg: &str, config: &StreamConfig) -> Result<Event, ParseError> {
        // Implementation goes here
        unimplemented!()
    }
}

#[derive(Deserialize, Clone)]
struct CoinbaseTrade {
    product_id: String,
    trade_id: String,
    price: Decimal,
    size: Decimal,
    time: String,
    side: String,
}
