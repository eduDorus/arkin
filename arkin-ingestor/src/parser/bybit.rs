use super::ExchangeParser;
use crate::events::{MarketEvent, Side, Tick, Trade};
use crate::market_config::{StreamConfig, StreamType};
use rust_decimal::Decimal;
use serde::Deserialize;

pub struct BybitParser;

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

impl ExchangeParser for BybitParser {
    fn parse(&self, msg: &str, config: &StreamConfig) -> Result<Option<MarketEvent>, super::ParseError> {
        let value: serde_json::Value =
            serde_json::from_str(msg).map_err(|e| super::ParseError::JsonParse(e.to_string()))?;

        // Bybit format: {"topic":"publicTrade.BTCUSDT","data":[...],"ts":...} OR {"topic":"tickers.BTCUSDT","data":{...},"ts":...}
        let data = value
            .get("data")
            .ok_or_else(|| super::ParseError::MissingField("data".to_string()))?;

        match config.stream_type {
            StreamType::Trades | StreamType::AggregateTrades => {
                // For trades, data is an array
                let data_array = data
                    .as_array()
                    .ok_or_else(|| super::ParseError::MissingField("data array".to_string()))?;

                if data_array.is_empty() {
                    return Ok(None);
                }

                let trade: BybitTrade = serde_json::from_value(data_array[0].clone())
                    .map_err(|e| super::ParseError::JsonParse(e.to_string()))?;

                Ok(Some(MarketEvent::Trade(Trade {
                    exchange: config.exchange.to_string(),
                    market: config.market_type.to_string(),
                    symbol: trade.symbol,
                    price: trade.price,
                    quantity: trade.volume,
                    side: match trade.side.to_lowercase().as_str() {
                        "buy" => Side::Buy,
                        "sell" => Side::Sell,
                        _ => return Err(super::ParseError::InvalidValue("side".to_string())),
                    },
                    timestamp: trade.timestamp,
                    trade_id: trade.trade_id,
                    is_maker: None,
                })))
            }
            StreamType::TickerRealtime | StreamType::Ticker24h => {
                // For ticker, data is an object
                let ticker: BybitTicker =
                    serde_json::from_value(data.clone()).map_err(|e| super::ParseError::JsonParse(e.to_string()))?;

                // Get ts from top level if not in ticker object
                let timestamp = value.get("ts").and_then(|v| v.as_u64()).unwrap_or(ticker.timestamp);

                Ok(Some(MarketEvent::Tick(Tick {
                    exchange: config.exchange.to_string(),
                    market: config.market_type.to_string(),
                    symbol: ticker.symbol,
                    bid: Some(ticker.bid_price),
                    ask: Some(ticker.ask_price),
                    bid_qty: Some(ticker.bid_qty),
                    ask_qty: Some(ticker.ask_qty),
                    last_price: Some(ticker.last_price),
                    high_24h: Some(ticker.high_24h),
                    low_24h: Some(ticker.low_24h),
                    volume_24h: Some(ticker.volume_24h),
                    volume_quote_24h: Some(ticker.turnover_24h),
                    timestamp,
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
            exchange: Exchange::Bybit,
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
        let parser = BybitParser;
        let config = create_config(StreamType::Trades, MarketType::Spot);
        let json = r#"{"topic":"publicTrade.BTCUSDT","ts":1761583474344,"type":"snapshot","data":[{"i":"2290000000923750370","T":1761583474343,"p":"115440.7","v":"0.00903","S":"Buy","seq":89200463112,"s":"BTCUSDT","BT":false,"RPI":false}]}"#;

        let result = parser.parse(json, &config).unwrap().unwrap();
        match result {
            MarketEvent::Trade(t) => {
                assert_eq!(t.symbol, "BTCUSDT");
                assert_eq!(t.price, Decimal::from_str_exact("115440.7").unwrap());
                assert_eq!(t.side, Side::Buy);
            }
            _ => panic!("Expected Trade event"),
        }
    }

    #[test]
    fn test_parse_ticker() {
        let parser = BybitParser;
        let config = create_config(StreamType::TickerRealtime, MarketType::Spot);
        let json = r#"{"topic":"tickers.BTCUSDT","ts":1761583474344,"type":"snapshot","cs":89200463112,"data":{"symbol":"BTCUSDT","lastPrice":"115440.7","highPrice24h":"116400","lowPrice24h":"112870.6","prevPrice24h":"113634","volume24h":"8066.85312","turnover24h":"927680413.88337912","price24hPcnt":"0.0159","usdIndexPrice":"115456.767193","bid1Price":"115440","bid1Size":"1.5","ask1Price":"115440.7","ask1Size":"2.5"}}"#;

        let result = parser.parse(json, &config).unwrap().unwrap();
        match result {
            MarketEvent::Tick(t) => {
                assert_eq!(t.symbol, "BTCUSDT");
                assert_eq!(t.last_price, Some(Decimal::from_str_exact("115440.7").unwrap()));
                assert_eq!(t.high_24h, Some(Decimal::from_str_exact("116400").unwrap()));
            }
            _ => panic!("Expected Tick event"),
        }
    }
}
