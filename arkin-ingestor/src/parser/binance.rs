use super::ExchangeParser;
use crate::events::{MarketEvent, Metric, MetricType, Side, Tick, Trade};
use crate::market_config::{StreamConfig, StreamType};
use rust_decimal::Decimal;
use serde::Deserialize;

pub struct BinanceParser;

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

impl ExchangeParser for BinanceParser {
    fn parse(&self, msg: &str, config: &StreamConfig) -> Result<Option<MarketEvent>, super::ParseError> {
        let value: serde_json::Value =
            serde_json::from_str(msg).map_err(|e| super::ParseError::JsonParse(e.to_string()))?;

        match config.stream_type {
            StreamType::AggregateTrades => {
                let trade: BinanceAggregateTrade =
                    serde_json::from_value(value).map_err(|e| super::ParseError::JsonParse(e.to_string()))?;
                Ok(Some(MarketEvent::Trade(Trade {
                    exchange: config.exchange.to_string(),
                    market: config.market_type.to_string(),
                    symbol: config.params.get("symbol").cloned().unwrap_or_default(),
                    price: trade.price,
                    quantity: trade.quantity,
                    side: if trade.is_buyer_maker {
                        Side::Sell
                    } else {
                        Side::Buy
                    },
                    timestamp: trade.trade_time,
                    trade_id: trade.trade_id.to_string(),
                    is_maker: Some(trade.is_buyer_maker),
                })))
            }
            StreamType::Ticker24h => {
                let ticker: BinanceTicker24h =
                    serde_json::from_value(value).map_err(|e| super::ParseError::JsonParse(e.to_string()))?;
                Ok(Some(MarketEvent::Tick(Tick {
                    exchange: config.exchange.to_string(),
                    market: config.market_type.to_string(),
                    symbol: ticker.symbol,
                    bid: Some(ticker.bid_price),
                    ask: Some(ticker.ask_price),
                    bid_qty: Some(ticker.bid_qty),
                    ask_qty: Some(ticker.ask_qty),
                    last_price: Some(ticker.last_price),
                    high_24h: Some(ticker.high_price),
                    low_24h: Some(ticker.low_price),
                    volume_24h: Some(ticker.volume),
                    volume_quote_24h: Some(ticker.volume_quote),
                    timestamp: ticker.event_time,
                })))
            }
            StreamType::MarkPrice => {
                let mark: BinanceMarkPrice =
                    serde_json::from_value(value).map_err(|e| super::ParseError::JsonParse(e.to_string()))?;
                Ok(Some(MarketEvent::Metric(Metric {
                    exchange: config.exchange.to_string(),
                    market: config.market_type.to_string(),
                    symbol: mark.symbol,
                    metric_type: MetricType::MarkPrice,
                    value: mark.mark_price,
                    metadata: None,
                    timestamp: mark.event_time,
                })))
            }
            _ => Err(super::ParseError::UnknownStreamType(config.stream_type.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market_config::{Exchange, MarketType};
    use std::collections::HashMap;

    fn create_config(stream_type: StreamType, market_type: MarketType) -> StreamConfig {
        StreamConfig {
            exchange: Exchange::Binance,
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
    fn test_parse_aggregate_trade() {
        let parser = BinanceParser;
        let config = create_config(StreamType::AggregateTrades, MarketType::Spot);
        let json = r#"{"e":"aggTrade","E":1234567890000,"s":"BNBBTC","a":12345,"p":"0.001","q":"100","T":1234567000,"m":false}"#;

        let result = parser.parse(json, &config).unwrap().unwrap();
        match result {
            MarketEvent::Trade(t) => {
                assert_eq!(t.price, Decimal::from_str_exact("0.001").unwrap());
                assert_eq!(t.quantity, Decimal::from(100));
                assert_eq!(t.side, Side::Buy);
            }
            _ => panic!("Expected Trade event"),
        }
    }

    #[test]
    fn test_parse_ticker() {
        let parser = BinanceParser;
        let config = create_config(StreamType::Ticker24h, MarketType::Spot);
        let json = r#"{"e":"24hrTicker","E":1234567890000,"s":"BNBBTC","c":"0.001","b":"0.0009","B":"1.5","a":"0.0011","A":"2.5","h":"116400","l":"112870","v":"8066","q":"927680000"}"#;

        let result = parser.parse(json, &config).unwrap().unwrap();
        match result {
            MarketEvent::Tick(t) => {
                assert_eq!(t.symbol, "BNBBTC");
                assert_eq!(t.last_price, Some(Decimal::from_str_exact("0.001").unwrap()));
            }
            _ => panic!("Expected Tick event"),
        }
    }

    #[test]
    fn test_parse_mark_price() {
        let parser = BinanceParser;
        let config = create_config(StreamType::MarkPrice, MarketType::Perpetual);
        let json = r#"{"e":"markPriceUpdate","E":1234567890000,"s":"BNBBTC","p":"0.002"}"#;

        let result = parser.parse(json, &config).unwrap().unwrap();
        match result {
            MarketEvent::Metric(m) => {
                assert_eq!(m.metric_type, MetricType::MarkPrice);
                assert_eq!(m.value, Decimal::from_str_exact("0.002").unwrap());
            }
            _ => panic!("Expected Metric event"),
        }
    }
}
