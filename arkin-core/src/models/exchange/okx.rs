use rust_decimal::Decimal;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use time::UtcDateTime;

use crate::prelude::*;

/// Trade message from OKX.
///
/// Represents a WebSocket message containing trade data from OKX.
/// Contains channel arguments and an array of individual trade executions.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxTradeMessage {
    /// Channel subscription arguments
    pub arg: OkxArg,
    /// Array of individual trade data
    pub data: Vec<OkxTrade>,
}

/// Open interest message from OKX.
///
/// Represents a WebSocket message containing open interest data from OKX.
/// Contains channel arguments and an array of individual open interest data points.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OkxOpenInterestMessage {
    /// Channel subscription arguments
    pub arg: OkxArg,
    /// Array of individual open interest data
    pub data: Vec<OkxOpenInterest>,
}

/// Channel subscription arguments for OKX.
///
/// Contains metadata about the WebSocket subscription that produced this message.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OkxArg {
    /// Channel name (e.g., "trades", "open-interest")
    pub channel: String,
    /// Instrument identifier (e.g., "ETH-BTC", "BTC-USDT-SWAP")
    #[serde(rename = "instId")]
    pub instrument: String,
}

/// Individual trade data from OKX.
///
/// Represents a single trade execution on OKX, including price, quantity,
/// and trade direction. The `count` field may be present for aggregated trades.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OkxTrade {
    /// Instrument identifier
    #[serde(rename = "instId")]
    pub instrument: String,
    /// Transaction timestamp (when the trade executed)
    #[serde(rename = "ts", with = "custom_serde::timestamp")]
    pub transaction_time: UtcDateTime,
    /// Unique trade identifier
    #[serde(rename = "tradeId")]
    pub trade_id: String,
    /// Execution price
    #[serde(rename = "px")]
    pub price: Decimal,
    /// Trade quantity
    #[serde(rename = "sz")]
    pub quantity: Decimal,
    /// Trade side ("buy" or "sell")
    pub side: String,
    /// Number of trades aggregated (optional, may be None)
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub count: Option<u64>,
}

/// Individual open interest data from OKX.
///
/// Represents the current open interest for a specific instrument on OKX,
/// including both contract count and currency-denominated values.
#[serde_as]
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OkxOpenInterest {
    /// Instrument identifier
    #[serde(rename = "instId")]
    pub instrument: String,
    /// Instrument type (e.g., "SWAP", "FUTURES")
    #[serde(rename = "instType")]
    pub instrument_type: String,
    /// Open interest in contracts
    #[serde_as(as = "DisplayFromStr")]
    pub oi: Decimal,
    /// Open interest in currency terms
    #[serde_as(as = "DisplayFromStr")]
    #[serde(rename = "oiCcy")]
    pub oi_ccy: Decimal,
    /// Timestamp when this data was recorded
    #[serde(rename = "ts", with = "custom_serde::timestamp")]
    pub timestamp: UtcDateTime,
}

impl OkxTrade {
    /// Convert this OKX trade to the unified exchange event data format.
    ///
    /// Transforms OKX-specific trade data into the standardized
    /// [`ExchangeEventData`] format used across all exchanges.
    ///
    /// The trade side is converted from OKX's string format ("buy"/"sell")
    /// to the unified [`MarketSide`] enum.
    pub fn to_unified(self) -> crate::models::exchange::ExchangeEventData {
        let side = match self.side.as_str() {
            "buy" => crate::MarketSide::Buy,
            "sell" => crate::MarketSide::Sell,
            _ => crate::MarketSide::Buy, // Default to buy if unknown
        };

        crate::models::exchange::ExchangeEventData::Trade(crate::models::exchange::TradeData {
            event_time: self.transaction_time,
            transaction_time: self.transaction_time,
            trade_id: self.trade_id,
            price: self.price,
            quantity: self.quantity,
            side,
            maker: false, // OKX doesn't provide maker information in trade data
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
    /// Returns the instrument identifier (e.g., "ETH-BTC", "BTC-USDT-SWAP") for this trade.
    pub fn venue_symbol(&self) -> &str {
        &self.instrument
    }
}

impl OkxOpenInterest {
    /// Convert this OKX open interest to the unified exchange event data format.
    ///
    /// Transforms OKX-specific open interest data into the standardized
    /// [`ExchangeEventData`] format used across all exchanges.
    ///
    /// Note: Open interest is not directly supported in the current unified
    /// event types, so this returns a placeholder Trade event for now.
    /// This may need to be updated when open interest support is added.
    pub fn to_unified(self) -> crate::models::exchange::ExchangeEventData {
        // TODO: Add OpenInterest variant to ExchangeEventData when needed
        // For now, this is a placeholder that creates a minimal trade event
        crate::models::exchange::ExchangeEventData::Trade(crate::models::exchange::TradeData {
            event_time: self.timestamp,
            transaction_time: self.timestamp,
            trade_id: format!("oi_{}_{}", self.instrument, self.timestamp.unix_timestamp()),
            price: self.oi_ccy,           // Using currency OI as price for now
            quantity: self.oi,            // Using contract OI as quantity
            side: crate::MarketSide::Buy, // Placeholder
            maker: false,
        })
    }

    /// Get the event time for this open interest data.
    ///
    /// Returns the timestamp when this open interest data was recorded.
    pub fn event_time(&self) -> time::UtcDateTime {
        self.timestamp
    }

    /// Get the venue symbol for instrument lookup.
    ///
    /// Returns the instrument identifier (e.g., "BTC-USD-SWAP") for this open interest data.
    pub fn venue_symbol(&self) -> &str {
        &self.instrument
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_okx_spot_trade() {
        let json_data = r#"{"arg":{"channel":"trades","instId":"ETH-BTC"},"data":[{"instId":"ETH-BTC","tradeId":"55597951","px":"0.03567","sz":"0.0205","side":"buy","ts":"1735689652105"}]}"#;
        let _ = serde_json::from_str::<OkxTradeMessage>(json_data).unwrap();
    }

    #[test]
    fn test_okx_perp_trade() {
        let json_data = r#"{"arg":{"channel":"trades","instId":"BTC-USDT-SWAP"},"data":[{"instId":"BTC-USDT-SWAP","tradeId":"1216801608","px":"93630","sz":"2.1","side":"buy","ts":"1735689659701","count":"1"}]}"#;
        let _ = serde_json::from_str::<OkxTradeMessage>(json_data).unwrap();
    }

    #[test]
    fn test_okx_open_interest() {
        let json_data = r#"{"arg":{"channel":"open-interest","instId":"BTC-USD-SWAP"},"data":[{"instId":"BTC-USD-SWAP","instType":"SWAP","oi":"8824028","oiCcy":"9450.4815189266069195","ts":"1735689601474"}]}"#;
        let _ = serde_json::from_str::<OkxOpenInterestMessage>(json_data).unwrap();
    }
}
