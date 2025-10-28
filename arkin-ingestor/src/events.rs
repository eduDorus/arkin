use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

/// Helper function to deserialize Decimal from both strings and numbers
fn deserialize_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Value {
        String(String),
        Number(serde_json::Number),
    }

    match Value::deserialize(deserializer)? {
        Value::String(s) => Decimal::from_str_exact(&s).map_err(serde::de::Error::custom),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(Decimal::from(i))
            } else if let Some(u) = n.as_u64() {
                Ok(Decimal::from(u))
            } else if let Some(f) = n.as_f64() {
                Decimal::from_f64_retain(f)
                    .ok_or_else(|| serde::de::Error::custom(format!("cannot convert {} to Decimal", f)))
            } else {
                Err(serde::de::Error::custom("number is not valid"))
            }
        }
    }
}

/// Helper function to deserialize optional Decimal
fn deserialize_optional_decimal<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Value {
        Null,
        String(String),
        Number(serde_json::Number),
    }

    match Value::deserialize(deserializer) {
        Ok(Value::Null) => Ok(None),
        Ok(Value::String(s)) => Decimal::from_str_exact(&s).map(Some).map_err(serde::de::Error::custom),
        Ok(Value::Number(n)) => {
            if let Some(i) = n.as_i64() {
                Ok(Some(Decimal::from(i)))
            } else if let Some(u) = n.as_u64() {
                Ok(Some(Decimal::from(u)))
            } else if let Some(f) = n.as_f64() {
                Decimal::from_f64_retain(f)
                    .map(Some)
                    .ok_or_else(|| serde::de::Error::custom(format!("cannot convert {} to Decimal", f)))
            } else {
                Err(serde::de::Error::custom("number is not valid"))
            }
        }
        Err(_) => Ok(None),
    }
}

/// Trade event - represents a single trade execution
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Trade {
    pub exchange: String,
    pub market: String,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub price: Decimal,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub quantity: Decimal,
    pub side: Side,
    pub timestamp: u64,
    pub trade_id: String,
    pub is_maker: Option<bool>,
}

/// Ticker event - represents current market snapshot
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Tick {
    pub exchange: String,
    pub market: String,
    pub symbol: String,
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    pub bid: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    pub ask: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    pub bid_qty: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    pub ask_qty: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    pub last_price: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    pub high_24h: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    pub low_24h: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    pub volume_24h: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    pub volume_quote_24h: Option<Decimal>,
    pub timestamp: u64,
}

/// Market metrics - funding rates, mark prices, liquidations, open interest
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Metric {
    pub exchange: String,
    pub market: String,
    pub symbol: String,
    pub metric_type: MetricType,
    #[serde(deserialize_with = "deserialize_decimal")]
    pub value: Decimal,
    pub metadata: Option<MetricMetadata>,
    pub timestamp: u64,
}

/// Types of metrics available
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    FundingRate,
    MarkPrice,
    IndexPrice,
    Liquidation,
    OpenInterest,
}

impl fmt::Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetricType::FundingRate => write!(f, "funding_rate"),
            MetricType::MarkPrice => write!(f, "mark_price"),
            MetricType::IndexPrice => write!(f, "index_price"),
            MetricType::Liquidation => write!(f, "liquidation"),
            MetricType::OpenInterest => write!(f, "open_interest"),
        }
    }
}

/// Additional metadata for metrics
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MetricMetadata {
    pub funding_time: Option<u64>,
    pub liquidation_side: Option<Side>,
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    pub liquidated_qty: Option<Decimal>,
    #[serde(default, deserialize_with = "deserialize_optional_decimal")]
    pub liquidation_price: Option<Decimal>,
    pub extra: Option<serde_json::Value>,
}

/// Trade side
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Buy,
    Sell,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Buy => write!(f, "buy"),
            Side::Sell => write!(f, "sell"),
        }
    }
}

impl Side {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "buy" | "b" | "buyer" | "long" => Some(Side::Buy),
            "sell" | "s" | "seller" | "short" => Some(Side::Sell),
            _ => None,
        }
    }
}

/// Unified event enum
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum MarketEvent {
    Trade(Trade),
    Tick(Tick),
    Metric(Metric),
}

impl MarketEvent {
    pub fn exchange(&self) -> &str {
        match self {
            MarketEvent::Trade(t) => &t.exchange,
            MarketEvent::Tick(t) => &t.exchange,
            MarketEvent::Metric(m) => &m.exchange,
        }
    }

    pub fn symbol(&self) -> &str {
        match self {
            MarketEvent::Trade(t) => &t.symbol,
            MarketEvent::Tick(t) => &t.symbol,
            MarketEvent::Metric(m) => &m.symbol,
        }
    }

    pub fn timestamp(&self) -> u64 {
        match self {
            MarketEvent::Trade(t) => t.timestamp,
            MarketEvent::Tick(t) => t.timestamp,
            MarketEvent::Metric(m) => m.timestamp,
        }
    }

    pub fn market(&self) -> &str {
        match self {
            MarketEvent::Trade(t) => &t.market,
            MarketEvent::Tick(t) => &t.market,
            MarketEvent::Metric(m) => &m.market,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_side_parsing() {
        assert_eq!(Side::from_str("buy"), Some(Side::Buy));
        assert_eq!(Side::from_str("BUY"), Some(Side::Buy));
        assert_eq!(Side::from_str("sell"), Some(Side::Sell));
    }

    #[test]
    fn test_market_event_accessors() {
        let trade = Trade {
            exchange: "binance".to_string(),
            market: "spot".to_string(),
            symbol: "BTC-USDT".to_string(),
            price: Decimal::from(100),
            quantity: Decimal::from(1),
            side: Side::Buy,
            timestamp: 1234567890,
            trade_id: "123".to_string(),
            is_maker: Some(false),
        };

        let event = MarketEvent::Trade(trade);
        assert_eq!(event.exchange(), "binance");
        assert_eq!(event.symbol(), "BTC-USDT");
        assert_eq!(event.timestamp(), 1234567890);
    }
}
