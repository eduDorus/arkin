mod binance;
mod bybit;
mod coinbase;
mod okx;

use crate::events::MarketEvent;
use crate::market_config::StreamConfig;
use std::fmt;

/// Core parser trait - each exchange implements this
pub trait ExchangeParser: Send + Sync {
    /// Parse a single message from the exchange
    /// Returns Ok(Some(event)) if parsing succeeded and produced an event
    /// Returns Ok(None) if the message is not the expected type (e.g., a control message)
    /// Returns Err if parsing failed
    fn parse(&self, msg: &str, config: &StreamConfig) -> Result<Option<MarketEvent>, ParseError>;
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
    pub fn get_parser(exchange: &str) -> Result<Box<dyn ExchangeParser>, ParseError> {
        match exchange.to_lowercase().as_str() {
            "binance" => Ok(Box::new(binance::BinanceParser)),
            "okx" => Ok(Box::new(okx::OkxParser)),
            "bybit" => Ok(Box::new(bybit::BybitParser)),
            "coinbase" => Ok(Box::new(coinbase::CoinbaseParser)),
            _ => Err(ParseError::UnknownExchange(exchange.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_factory_known_exchanges() {
        assert!(ParserFactory::get_parser("binance").is_ok());
        assert!(ParserFactory::get_parser("okx").is_ok());
        assert!(ParserFactory::get_parser("bybit").is_ok());
        assert!(ParserFactory::get_parser("coinbase").is_ok());
    }

    #[test]
    fn test_parser_factory_unknown_exchange() {
        match ParserFactory::get_parser("unknown") {
            Err(ParseError::UnknownExchange(e)) => assert_eq!(e, "unknown"),
            _ => panic!("Expected UnknownExchange error"),
        }
    }

    #[test]
    fn test_parse_error_display() {
        let err = ParseError::MissingField("price".to_string());
        assert_eq!(err.to_string(), "Missing field: price");
    }
}
