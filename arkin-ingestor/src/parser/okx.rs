use super::ExchangeParser;
use crate::events::{MarketEvent, Metric, MetricType, Side, Tick, Trade};
use crate::market_config::{StreamConfig, StreamType};
use rust_decimal::Decimal;
use serde::Deserialize;

pub struct OkxParser;

#[derive(Deserialize)]
struct OkxTrade {
    #[serde(rename = "instId")]
    inst_id: String,
    #[serde(rename = "px")]
    price: Decimal,
    #[serde(rename = "sz")]
    quantity: Decimal,
    side: String,
    #[serde(rename = "ts")]
    timestamp_str: String,
    #[serde(rename = "tradeId")]
    trade_id: String,
}

#[derive(Deserialize)]
struct OkxTicker {
    #[serde(rename = "instId")]
    inst_id: String,
    #[serde(rename = "bidPx")]
    bid_px: Decimal,
    #[serde(rename = "bidSz")]
    bid_sz: Decimal,
    #[serde(rename = "askPx")]
    ask_px: Decimal,
    #[serde(rename = "askSz")]
    ask_sz: Decimal,
    #[serde(rename = "last")]
    last_price: Decimal,
    #[serde(rename = "high24h")]
    high_24h: Decimal,
    #[serde(rename = "low24h")]
    low_24h: Decimal,
    #[serde(rename = "vol24h")]
    vol_24h: Decimal,
    #[serde(rename = "volCcy24h")]
    vol_ccy_24h: Decimal,
    #[serde(rename = "ts")]
    timestamp_str: String,
}

#[derive(Deserialize)]
struct OkxFundingRate {
    #[serde(rename = "instId")]
    inst_id: String,
    #[serde(rename = "fundingRate")]
    funding_rate: Decimal,
    #[serde(rename = "ts")]
    timestamp_str: String,
}

impl ExchangeParser for OkxParser {
    fn parse(&self, msg: &str, config: &StreamConfig) -> Result<Option<MarketEvent>, super::ParseError> {
        let value: serde_json::Value =
            serde_json::from_str(msg).map_err(|e| super::ParseError::JsonParse(e.to_string()))?;

        // OKX wraps data in {"arg":{...},"data":[...]}
        let data_array = value
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| super::ParseError::MissingField("data".to_string()))?;

        if data_array.is_empty() {
            return Ok(None);
        }

        let data = &data_array[0];

        match config.stream_type {
            StreamType::Trades | StreamType::AggregateTrades => {
                let trade: OkxTrade =
                    serde_json::from_value(data.clone()).map_err(|e| super::ParseError::JsonParse(e.to_string()))?;
                let timestamp = trade
                    .timestamp_str
                    .parse::<u64>()
                    .map_err(|_| super::ParseError::InvalidValue("timestamp".to_string()))?;
                Ok(Some(MarketEvent::Trade(Trade {
                    exchange: config.exchange.to_string(),
                    market: config.market_type.to_string(),
                    symbol: trade.inst_id.replace("-", ""),
                    price: trade.price,
                    quantity: trade.quantity,
                    side: match trade.side.to_lowercase().as_str() {
                        "buy" => Side::Buy,
                        "sell" => Side::Sell,
                        _ => return Err(super::ParseError::InvalidValue("side".to_string())),
                    },
                    timestamp,
                    trade_id: trade.trade_id,
                    is_maker: None,
                })))
            }
            StreamType::TickerRealtime | StreamType::Ticker24h => {
                let ticker: OkxTicker =
                    serde_json::from_value(data.clone()).map_err(|e| super::ParseError::JsonParse(e.to_string()))?;
                let timestamp = ticker
                    .timestamp_str
                    .parse::<u64>()
                    .map_err(|_| super::ParseError::InvalidValue("timestamp".to_string()))?;
                Ok(Some(MarketEvent::Tick(Tick {
                    exchange: config.exchange.to_string(),
                    market: config.market_type.to_string(),
                    symbol: ticker.inst_id.replace("-", ""),
                    bid: Some(ticker.bid_px),
                    ask: Some(ticker.ask_px),
                    bid_qty: Some(ticker.bid_sz),
                    ask_qty: Some(ticker.ask_sz),
                    last_price: Some(ticker.last_price),
                    high_24h: Some(ticker.high_24h),
                    low_24h: Some(ticker.low_24h),
                    volume_24h: Some(ticker.vol_24h),
                    volume_quote_24h: Some(ticker.vol_ccy_24h),
                    timestamp,
                })))
            }
            StreamType::FundingRate => {
                let rate: OkxFundingRate =
                    serde_json::from_value(data.clone()).map_err(|e| super::ParseError::JsonParse(e.to_string()))?;
                let timestamp = rate
                    .timestamp_str
                    .parse::<u64>()
                    .map_err(|_| super::ParseError::InvalidValue("timestamp".to_string()))?;
                Ok(Some(MarketEvent::Metric(Metric {
                    exchange: config.exchange.to_string(),
                    market: config.market_type.to_string(),
                    symbol: rate.inst_id.replace("-", ""),
                    metric_type: MetricType::FundingRate,
                    value: rate.funding_rate,
                    metadata: None,
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
            exchange: Exchange::Okx,
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
        let parser = OkxParser;
        let config = create_config(StreamType::Trades, MarketType::Spot);
        let json = r#"{"arg":{"channel":"trades","instId":"BTC-USDT"},"data":[{"instId":"BTC-USDT","tradeId":"835232870","px":"115451.2","sz":"0.0000305","side":"sell","ts":"1761583471606"}]}"#;

        let result = parser.parse(json, &config).unwrap().unwrap();
        match result {
            MarketEvent::Trade(t) => {
                assert_eq!(t.symbol, "BTCUSDT");
                assert_eq!(t.price, Decimal::from_str_exact("115451.2").unwrap());
                assert_eq!(t.side, Side::Sell);
            }
            _ => panic!("Expected Trade event"),
        }
    }

    #[test]
    fn test_parse_ticker() {
        let parser = OkxParser;
        let config = create_config(StreamType::TickerRealtime, MarketType::Spot);
        let json = r#"{"arg":{"channel":"tickers","instId":"BTC-USDT"},"data":[{"instType":"SPOT","instId":"BTC-USDT","last":"115450","lastSz":"0.01299737","askPx":"115450.1","askSz":"2.57408132","bidPx":"115450","bidSz":"1.62542563","open24h":"113625.5","high24h":"116400","low24h":"112900","sodUtc0":"114555.7","sodUtc8":"114959.9","volCcy24h":"887468830.390342052","vol24h":"7715.62528481","ts":"1761583471970"}]}"#;

        let result = parser.parse(json, &config).unwrap().unwrap();
        match result {
            MarketEvent::Tick(t) => {
                assert_eq!(t.symbol, "BTCUSDT");
                assert_eq!(t.bid, Some(Decimal::from_str_exact("115450").unwrap()));
            }
            _ => panic!("Expected Tick event"),
        }
    }

    #[test]
    fn test_parse_funding_rate() {
        let parser = OkxParser;
        let config = create_config(StreamType::FundingRate, MarketType::Perpetual);
        let json = r#"{"arg":{"channel":"funding-rate","instId":"BTC-USDT-SWAP"},"data":[{"instId":"BTC-USDT-SWAP","fundingRate":"0.0000464562120794","fundingTime":"1761609600000","ts":"1761583450140"}]}"#;

        let result = parser.parse(json, &config).unwrap().unwrap();
        match result {
            MarketEvent::Metric(m) => {
                assert_eq!(m.symbol, "BTCUSDTSWAP");
                assert_eq!(m.metric_type, MetricType::FundingRate);
            }
            _ => panic!("Expected Metric event"),
        }
    }
}
