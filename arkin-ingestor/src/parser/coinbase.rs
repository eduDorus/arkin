use super::ExchangeParser;
use crate::events::{MarketEvent, Side, Tick, Trade};
use crate::market_config::{StreamConfig, StreamType};
use rust_decimal::Decimal;
use serde::Deserialize;

pub struct CoinbaseParser;

#[derive(Deserialize, Clone)]
struct CoinbaseTrade {
    product_id: String,
    trade_id: String,
    price: Decimal,
    size: Decimal,
    time: String,
    side: String,
}

impl ExchangeParser for CoinbaseParser {
    fn parse(&self, msg: &str, config: &StreamConfig) -> Result<Option<MarketEvent>, super::ParseError> {
        let value: serde_json::Value =
            serde_json::from_str(msg).map_err(|e| super::ParseError::JsonParse(e.to_string()))?;

        // Coinbase format: {"channel":"market_trades",...,"events":[{"type":"snapshot","trades":[...]}]}
        let events = value
            .get("events")
            .and_then(|v| v.as_array())
            .ok_or_else(|| super::ParseError::MissingField("events".to_string()))?;

        if events.is_empty() {
            return Ok(None);
        }

        let event = &events[0];
        let trades = event
            .get("trades")
            .and_then(|v| v.as_array())
            .ok_or_else(|| super::ParseError::MissingField("trades".to_string()))?;

        if trades.is_empty() {
            return Ok(None);
        }

        let trade_data: CoinbaseTrade =
            serde_json::from_value(trades[0].clone()).map_err(|e| super::ParseError::JsonParse(e.to_string()))?;

        match config.stream_type {
            StreamType::Trades | StreamType::AggregateTrades => {
                let timestamp = parse_rfc3339_to_ms(&trade_data.time)?;
                Ok(Some(MarketEvent::Trade(Trade {
                    exchange: config.exchange.to_string(),
                    market: config.market_type.to_string(),
                    symbol: trade_data.product_id,
                    price: trade_data.price,
                    quantity: trade_data.size,
                    side: match trade_data.side.to_uppercase().as_str() {
                        "BUY" => Side::Buy,
                        "SELL" => Side::Sell,
                        _ => return Err(super::ParseError::InvalidValue("side".to_string())),
                    },
                    timestamp,
                    trade_id: trade_data.trade_id,
                    is_maker: None,
                })))
            }
            StreamType::TickerRealtime | StreamType::Ticker24h => {
                let timestamp = parse_rfc3339_to_ms(&trade_data.time)?;
                Ok(Some(MarketEvent::Tick(Tick {
                    exchange: config.exchange.to_string(),
                    market: config.market_type.to_string(),
                    symbol: trade_data.product_id,
                    bid: None,
                    ask: None,
                    bid_qty: None,
                    ask_qty: None,
                    last_price: Some(trade_data.price),
                    high_24h: None,
                    low_24h: None,
                    volume_24h: None,
                    volume_quote_24h: None,
                    timestamp,
                })))
            }
            _ => Err(super::ParseError::UnknownStreamType(config.stream_type.to_string())),
        }
    }
}

/// Convert RFC3339 timestamp string to Unix milliseconds
/// Format: "2025-10-27T16:44:39.600153Z"
fn parse_rfc3339_to_ms(time_str: &str) -> Result<u64, super::ParseError> {
    // Manual RFC3339 parsing to avoid external dependency
    let parts: Vec<&str> = time_str.split('T').collect();
    if parts.len() != 2 {
        return Err(super::ParseError::InvalidValue(format!(
            "Invalid timestamp format: {}",
            time_str
        )));
    }

    let date_part = parts[0];
    let time_part = parts[1].trim_end_matches('Z');

    // Parse date (YYYY-MM-DD)
    let date_components: Vec<&str> = date_part.split('-').collect();
    if date_components.len() != 3 {
        return Err(super::ParseError::InvalidValue(format!("Invalid date format: {}", date_part)));
    }

    let year: i32 = date_components[0]
        .parse()
        .map_err(|_| super::ParseError::InvalidValue("year".to_string()))?;
    let month: u32 = date_components[1]
        .parse()
        .map_err(|_| super::ParseError::InvalidValue("month".to_string()))?;
    let day: u32 = date_components[2]
        .parse()
        .map_err(|_| super::ParseError::InvalidValue("day".to_string()))?;

    // Parse time (HH:MM:SS.ffffff)
    let time_components: Vec<&str> = time_part.split(':').collect();
    if time_components.len() != 3 {
        return Err(super::ParseError::InvalidValue("time".to_string()));
    }

    let hour: u32 = time_components[0]
        .parse()
        .map_err(|_| super::ParseError::InvalidValue("hour".to_string()))?;
    let minute: u32 = time_components[1]
        .parse()
        .map_err(|_| super::ParseError::InvalidValue("minute".to_string()))?;

    let sec_parts: Vec<&str> = time_components[2].split('.').collect();
    let second: u32 = sec_parts[0]
        .parse()
        .map_err(|_| super::ParseError::InvalidValue("second".to_string()))?;

    let millis: u32 = if sec_parts.len() == 2 {
        let frac_str = format!("{:0<3}", sec_parts[1]);
        frac_str[..3]
            .parse()
            .map_err(|_| super::ParseError::InvalidValue("fraction".to_string()))?
    } else {
        0
    };

    // Calculate days since epoch
    let mut days = 0i64;

    // Days for years
    for y in 1970..year {
        days += if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) {
            366
        } else {
            365
        };
    }

    // Days for months in current year
    let is_leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let month_days = [31, if is_leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    for m in 1..month as usize {
        days += month_days[m - 1] as i64;
    }

    // Days in current month
    days += (day - 1) as i64;

    // Convert to seconds
    let total_seconds = days * 86400 + hour as i64 * 3600 + minute as i64 * 60 + second as i64;

    // Convert to milliseconds
    let total_millis = total_seconds * 1000 + millis as i64;

    Ok(total_millis as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market_config::{Exchange, MarketType};
    use std::collections::HashMap;

    fn create_config(stream_type: StreamType, market_type: MarketType) -> StreamConfig {
        StreamConfig {
            exchange: Exchange::Coinbase,
            stream_type,
            market_type,
            url: "ws://localhost".to_string(),
            subscription_message: "{}".to_string(),
            update_frequency_ms: Some(100),
            description: "test".to_string(),
            params: HashMap::new(),
        }
    }

    #[test]
    fn test_parse_trade() {
        let parser = CoinbaseParser;
        let config = create_config(StreamType::Trades, MarketType::Spot);
        let json = r#"{"channel":"market_trades","client_id":"","timestamp":"2025-10-27T16:44:40.175203666Z","sequence_num":0,"events":[{"type":"snapshot","trades":[{"product_id":"BTC-USD","trade_id":"892262231","price":"115445.28","size":"0.00015604","time":"2025-10-27T16:44:39.600153Z","side":"BUY"}]}]}"#;

        let result = parser.parse(json, &config).unwrap().unwrap();
        match result {
            MarketEvent::Trade(t) => {
                assert_eq!(t.symbol, "BTC-USD");
                assert_eq!(t.price, Decimal::from_str_exact("115445.28").unwrap());
                assert_eq!(t.quantity, Decimal::from_str_exact("0.00015604").unwrap());
                assert_eq!(t.side, Side::Buy);
            }
            _ => panic!("Expected Trade event"),
        }
    }

    #[test]
    fn test_parse_ticker() {
        let parser = CoinbaseParser;
        let config = create_config(StreamType::TickerRealtime, MarketType::Spot);
        let json = r#"{"channel":"market_trades","client_id":"","timestamp":"2025-10-27T16:44:40.175203666Z","sequence_num":0,"events":[{"type":"snapshot","trades":[{"product_id":"BTC-USD","trade_id":"892262231","price":"115445.28","size":"0.00015604","time":"2025-10-27T16:44:39.600153Z","side":"BUY"}]}]}"#;

        let result = parser.parse(json, &config).unwrap().unwrap();
        match result {
            MarketEvent::Tick(t) => {
                assert_eq!(t.symbol, "BTC-USD");
                assert_eq!(t.last_price, Some(Decimal::from_str_exact("115445.28").unwrap()));
            }
            _ => panic!("Expected Tick event"),
        }
    }

    #[test]
    fn test_parse_rfc3339_timestamp() {
        let result = parse_rfc3339_to_ms("2025-10-27T16:44:39.600153Z");
        assert!(result.is_ok());
        let ms = result.unwrap();
        assert!(ms > 0);
    }
}
