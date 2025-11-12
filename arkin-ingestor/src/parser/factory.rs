use async_trait::async_trait;
use std::{fmt, sync::Arc};

use arkin_core::prelude::*;

/// Core parser trait - each exchange implements this
#[async_trait]
pub trait VenueParser: Send + Sync {
    /// Parse a single message from the exchange
    async fn parse(
        &self,
        channel: Channel,
        msg: &str,
        persistence: &Arc<dyn PersistenceReader>,
    ) -> Result<Event, ParseError>;
}

/// Parser error types
#[derive(Clone, Debug)]
pub enum ParseError {
    JsonParse(String),
    MissingField(String),
    InvalidValue(String),
    UnknownExchange(String),
    UnknownStreamType(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::JsonParse(e) => write!(f, "JSON parse error: {}", e),
            ParseError::MissingField(field) => write!(f, "Missing field: {}", field),
            ParseError::InvalidValue(v) => write!(f, "Invalid value: {}", v),
            ParseError::UnknownExchange(e) => write!(f, "Unknown exchange: {}", e),
            ParseError::UnknownStreamType(s) => write!(f, "Unknown stream type: {}", s),
        }
    }
}

impl std::error::Error for ParseError {}

/// Central parser factory - knows how to get the right parser for each exchange
pub struct ParserFactory;

impl ParserFactory {
    /// Get the appropriate parser for an exchange
    pub fn get_parser(venue: VenueName, instrument_type: InstrumentType) -> Result<Box<dyn VenueParser>, ParseError> {
        match (venue, instrument_type) {
            // (VenueName::Binance, InstrumentType::Spot) => Ok(Box::new(binance_spot::BinanceSpotParser)),
            // (VenueName::Bybit, InstrumentType::Spot) => Ok(Box::new(bybit_spot::BybitSpotParser)),
            // (VenueName::Coinbase, InstrumentType::Spot) => Ok(Box::new(coinbase_spot::CoinbaseSpotParser)),
            // (VenueName::Okx, InstrumentType::Spot) => Ok(Box::new(okx_spot::OkxSpotParser)),
            _ => Err(ParseError::UnknownExchange(format!(
                "No parser for venue {:?} and instrument type {:?}",
                venue, instrument_type
            ))),
        }
    }
}
