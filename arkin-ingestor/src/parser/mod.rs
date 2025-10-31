mod binance_spot;
mod bybit_spot;
mod coinbase_spot;
mod okx_spot;

use arkin_core::Event;

use crate::{market_config::StreamConfig, registry::VenueName, MarketType};
use std::fmt;

/// Core parser trait - each exchange implements this
pub trait VenueParser: Send + Sync {
    /// Parse a single message from the exchange
    /// Returns Ok(Some(event)) if parsing succeeded and produced an event
    /// Returns Ok(None) if the message is not the expected type (e.g., a control message)
    /// Returns Err if parsing failed
    fn parse(&self, msg: &str, config: &StreamConfig) -> Result<Event, ParseError>;
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
    pub fn get_parser(venue: VenueName, market_type: MarketType) -> Result<Box<dyn VenueParser>, ParseError> {
        match (venue, market_type) {
            (VenueName::Binance, MarketType::Spot) => Ok(Box::new(binance_spot::BinanceSpotParser)),
            (VenueName::Bybit, MarketType::Spot) => Ok(Box::new(bybit_spot::BybitSpotParser)),
            (VenueName::Coinbase, MarketType::Spot) => Ok(Box::new(coinbase_spot::CoinbaseSpotParser)),
            (VenueName::Okx, MarketType::Spot) => Ok(Box::new(okx_spot::OkxSpotParser)),
            _ => Err(ParseError::UnknownExchange(format!(
                "No parser for venue {:?} and market type {:?}",
                venue, market_type
            ))),
        }
    }
}
