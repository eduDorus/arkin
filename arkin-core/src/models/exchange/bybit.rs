use rust_decimal::Decimal;
use serde::Deserialize;
use time::UtcDateTime;

use crate::prelude::*;

/// Public trade message from Bybit.
///
/// Represents a WebSocket message containing public trade data from Bybit.
/// Contains metadata about the message and an array of individual trades.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BybitTradeMessage {
    /// Topic/channel identifier (e.g., "publicTrade.BTCUSDT")
    pub topic: String,
    /// Message timestamp
    pub ts: i64,
    /// Message type (e.g., "snapshot", "delta")
    #[serde(rename = "type")]
    pub type_field: String,
    /// Array of individual trade data
    pub data: Vec<BybitTrade>,
}

/// Individual trade data from Bybit.
///
/// Represents a single trade execution on Bybit, including trade direction,
/// price, quantity, and additional metadata like block trade indicators.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BybitTrade {
    /// Trading pair symbol
    #[serde(rename = "s")]
    pub instrument: String,
    /// Transaction timestamp (when the trade executed)
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: UtcDateTime,
    /// Unique trade identifier
    #[serde(rename = "i")]
    pub trade_id: String,
    /// Execution price
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Trade quantity
    #[serde(rename = "v")]
    pub quantity: Decimal,
    /// Trade side ("Buy" or "Sell")
    #[serde(rename = "S")]
    pub side: String,
    /// Whether this was a block trade
    #[serde(rename = "BT")]
    pub block_trade: bool,
    /// Tick direction (perpetuals only: PlusTick, ZeroPlusTick, etc.)
    #[serde(rename = "L", default)]
    pub tick_direction: Option<String>,
}

impl BybitTrade {
    /// Convert this Bybit trade to the unified exchange event data format.
    ///
    /// Transforms Bybit-specific trade data into the standardized
    /// [`ExchangeEventData`] format used across all exchanges.
    ///
    /// The trade side is converted from Bybit's string format ("Buy"/"Sell")
    /// to the unified [`MarketSide`] enum.
    pub fn to_unified(self) -> crate::models::exchange::ExchangeEventData {
        let side = match self.side.as_str() {
            "Buy" => crate::MarketSide::Buy,
            "Sell" => crate::MarketSide::Sell,
            _ => crate::MarketSide::Buy, // Default to buy if unknown
        };

        crate::models::exchange::ExchangeEventData::Trade(crate::models::exchange::TradeData {
            event_time: self.transaction_time,
            transaction_time: self.transaction_time,
            trade_id: self.trade_id,
            price: self.price,
            quantity: self.quantity,
            side,
            maker: false, // Bybit doesn't provide maker information in public trade data
        })
    }

    /// Get the event time for this trade.
    ///
    /// Returns the transaction timestamp when this trade was executed.
    pub fn event_time(&self) -> time::UtcDateTime {
        self.transaction_time
    }

    /// Get the venue symbol for instrument lookup.
    ///
    /// Returns the trading pair symbol (e.g., "BTCUSDT") for this trade.
    pub fn venue_symbol(&self) -> &str {
        &self.instrument
    }
}

/// Ticker message from Bybit.
///
/// Represents a WebSocket message containing ticker data from Bybit.
/// Contains metadata about the message and ticker data.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BybitTickerMessage {
    /// Topic/channel identifier (e.g., "tickers.BTCUSDT")
    pub topic: String,
    /// Message type (e.g., "snapshot", "delta")
    #[serde(rename = "type")]
    pub type_field: String,
    /// Ticker data
    pub data: BybitTickerData,
    /// Timestamp
    pub ts: i64,
}

/// Individual ticker data from Bybit.
///
/// Represents ticker information including best bid/ask prices and quantities.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BybitTickerData {
    /// Trading pair symbol
    pub symbol: String,
    /// Best bid price
    #[serde(rename = "bid1Price")]
    pub bid_price: Decimal,
    /// Bid quantity
    #[serde(rename = "bid1Size")]
    pub bid_quantity: Decimal,
    /// Best ask price
    #[serde(rename = "ask1Price")]
    pub ask_price: Decimal,
    /// Ask quantity
    #[serde(rename = "ask1Size")]
    pub ask_quantity: Decimal,
}

impl BybitTickerData {
    /// Get the venue symbol for instrument lookup.
    ///
    /// Returns the trading pair symbol (e.g., "BTCUSDT") for this ticker data.
    pub fn venue_symbol(&self) -> &str {
        &self.symbol
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bybit_spot_trade() {
        let json_data = r#"{"topic":"publicTrade.BTCUSDT","ts":1735689653900,"type":"snapshot","data":[{"i":"2290000000587387108","T":1735689653898,"p":"93611.04","v":"0.000725","S":"Buy","s":"BTCUSDT","BT":false},{"i":"2290000000587387109","T":1735689653898,"p":"93611.77","v":"0.001378","S":"Buy","s":"BTCUSDT","BT":false},{"i":"2290000000587387110","T":1735689653898,"p":"93612.31","v":"0.018594","S":"Buy","s":"BTCUSDT","BT":false},{"i":"2290000000587387111","T":1735689653898,"p":"93612.51","v":"0.03205","S":"Buy","s":"BTCUSDT","BT":false},{"i":"2290000000587387112","T":1735689653898,"p":"93612.93","v":"0.0001","S":"Buy","s":"BTCUSDT","BT":false}]}"#;
        let _ = serde_json::from_str::<BybitTradeMessage>(json_data).unwrap();
    }

    #[test]
    fn test_bybit_perp_trade() {
        let json_data = r#"{"topic":"publicTrade.BTCUSDT","type":"snapshot","ts":1735689654138,"data":[{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93580.00","L":"PlusTick","i":"079fa26c-36af-5007-b4be-23079b78761f","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.200","p":"93580.00","L":"ZeroPlusTick","i":"b412cc85-2712-5387-bd55-8d4d6f6b6450","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.160","p":"93580.00","L":"ZeroPlusTick","i":"758b3ab3-7cec-510e-892a-c14d345080cd","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.021","p":"93580.10","L":"PlusTick","i":"85c3d15e-dd32-5a9f-8121-0f669bf88731","BT":false},{"T":1735689654137,"s":"BTCUSDT","S":"Buy","v":"0.001","p":"93580.20","L":"PlusTick","i":"4fc17326-6cea-5f30-881c-713860ffe548","BT":false}]}"#;
        let _ = serde_json::from_str::<BybitTradeMessage>(json_data).unwrap();
    }

    #[test]
    fn test_bybit_ticker() {
        let json_data = r#"{"topic":"tickers.BTCUSDT","type":"snapshot","ts":1735689654138,"data":{"symbol":"BTCUSDT","bid1Price":"93580.00","bid1Size":"0.001","ask1Price":"93581.00","ask1Size":"0.002","ts":1735689654138}}"#;
        let _ = serde_json::from_str::<BybitTickerMessage>(json_data).unwrap();
    }
}
