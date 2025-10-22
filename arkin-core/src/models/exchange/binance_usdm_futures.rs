#![allow(unused)]
use std::fmt;

use rust_decimal::Decimal;
use serde::Deserialize;
use time::UtcDateTime;

use crate::prelude::*;

/// Raw market data events from Binance exchanges.
///
/// This enum handles both historical data from Tardis (wrapped in stream format)
/// and live data directly from Binance's WebSocket API.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum BinanceMarketEvent {
    /// Historical data format from Tardis (includes stream wrapper)
    Stream(BinanceSwapsStreamEvent),
    /// Live data format directly from Binance exchange
    Direct(BinanceSwapsEvent),
}

/// Stream wrapper for historical data from Tardis.
///
/// Tardis provides historical market data with a stream wrapper around
/// the actual Binance event data.
#[derive(Debug, Clone)]
pub struct BinanceSwapsStreamEvent {
    /// Stream identifier (e.g., "btcusdt@aggTrade")
    pub stream: String,
    /// The actual Binance market event data
    pub data: BinanceSwapsEvent,
}

impl<'de> Deserialize<'de> for BinanceSwapsStreamEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct StreamEvent {
            stream: String,
            #[serde(rename = "generated")]
            _generated: Option<bool>,
            data: serde_json::Value,
        }

        let event = StreamEvent::deserialize(deserializer)?;

        // Determine event type from stream name pattern
        let event_type = if event.stream.contains("@trade") {
            // Individual trade
            let data: BinanceSwapsTradeData = serde_json::from_value(event.data).map_err(serde::de::Error::custom)?;
            BinanceSwapsEvent::Trade(data)
        } else if event.stream.contains("@aggTrade") {
            // Aggregated trade
            let data: BinanceSwapsAggTradeData =
                serde_json::from_value(event.data).map_err(serde::de::Error::custom)?;
            BinanceSwapsEvent::AggTrade(data)
        } else if event.stream.contains("@depth") {
            // Order book update
            let data: BinanceSwapsBookData = serde_json::from_value(event.data).map_err(serde::de::Error::custom)?;
            BinanceSwapsEvent::Book(data)
        } else if event.stream.contains("@bookTicker") {
            // Best bid/ask ticker
            let data: BinanceSwapsTickData = serde_json::from_value(event.data).map_err(serde::de::Error::custom)?;
            BinanceSwapsEvent::Tick(data)
        } else if event.stream.contains("@openInterest") {
            // Open interest update
            let data: BinanceSwapsOpenInterestData =
                serde_json::from_value(event.data).map_err(serde::de::Error::custom)?;
            BinanceSwapsEvent::OpenInterest(data)
        } else {
            return Err(serde::de::Error::custom(format!("Unknown stream type: {}", event.stream)));
        };

        Ok(BinanceSwapsStreamEvent {
            stream: event.stream,
            data: event_type,
        })
    }
}

/// Tagged enum for Binance market events.
///
/// Represents all possible market data event types that can be received
/// from Binance's WebSocket API, identified by the "e" (event type) field.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "e")]
pub enum BinanceSwapsEvent {
    /// Individual trade execution
    #[serde(rename = "trade")]
    Trade(BinanceSwapsTradeData),
    /// Aggregated trade (combines multiple individual trades)
    #[serde(rename = "aggTrade")]
    AggTrade(BinanceSwapsAggTradeData),
    /// Order book depth update
    #[serde(rename = "depthUpdate")]
    Book(BinanceSwapsBookData),
    /// Best bid/ask price update
    #[serde(rename = "bookTicker")]
    Tick(BinanceSwapsTickData),
    /// Open interest update
    #[serde(rename = "openInterest")]
    OpenInterest(BinanceSwapsOpenInterestData),
}

impl BinanceSwapsEvent {
    /// Convert this Binance event to the unified exchange event data format.
    ///
    /// This method transforms Binance-specific event data into the standardized
    /// [`ExchangeEventData`] enum that works across all exchanges.
    ///
    /// The conversion handles different event types:
    /// - Individual trades are converted to [`ExchangeEventData::Trade`]
    /// - Aggregated trades become [`ExchangeEventData::AggTrade`]
    /// - Order book updates become [`ExchangeEventData::BookUpdate`]
    /// - Ticker updates become [`ExchangeEventData::Tick`]
    pub fn to_unified(self) -> crate::models::exchange::ExchangeEventData {
        match self {
            BinanceSwapsEvent::Trade(trade) => {
                let side = if trade.trade_type == "MARKET" {
                    // For Binance, we can't determine side from trade_type, default to Buy
                    // In practice, this would need more context or different field
                    crate::MarketSide::Buy
                } else {
                    crate::MarketSide::Buy // Placeholder
                };
                crate::models::exchange::ExchangeEventData::Trade(crate::models::exchange::TradeData {
                    event_time: trade.event_time,
                    transaction_time: trade.transaction_time,
                    trade_id: trade.trade_id.to_string(),
                    price: trade.price,
                    quantity: trade.quantity,
                    side,
                    maker: trade.maker,
                })
            }
            BinanceSwapsEvent::AggTrade(agg_trade) => {
                let side = if agg_trade.maker {
                    crate::MarketSide::Sell
                } else {
                    crate::MarketSide::Buy
                };
                crate::models::exchange::ExchangeEventData::AggTrade(crate::models::exchange::AggTradeData {
                    event_time: agg_trade.event_time,
                    transaction_time: agg_trade.transaction_time,
                    trade_id: agg_trade.agg_trade_id,
                    first_trade_id: agg_trade.first_trade_id,
                    last_trade_id: agg_trade.last_trade_id,
                    price: agg_trade.price,
                    quantity: agg_trade.quantity,
                    side,
                    maker: agg_trade.maker,
                })
            }
            BinanceSwapsEvent::Book(book) => {
                let bids = book
                    .bids
                    .into_iter()
                    .map(|level| crate::models::exchange::BookLevel {
                        price: level.price,
                        quantity: level.quantity,
                    })
                    .collect();
                let asks = book
                    .asks
                    .into_iter()
                    .map(|level| crate::models::exchange::BookLevel {
                        price: level.price,
                        quantity: level.quantity,
                    })
                    .collect();
                crate::models::exchange::ExchangeEventData::BookUpdate(crate::models::exchange::BookUpdateData {
                    event_time: book.event_time,
                    transaction_time: book.transaction_time,
                    first_update_id: book.first_update_id,
                    final_update_id: book.final_update_id,
                    last_final_update_id: book.last_final_update_id,
                    bids,
                    asks,
                })
            }
            BinanceSwapsEvent::Tick(tick) => {
                crate::models::exchange::ExchangeEventData::Tick(crate::models::exchange::TickData {
                    event_time: tick.event_time,
                    transaction_time: tick.transaction_time,
                    update_id: tick.update_id,
                    bid_price: tick.bid_price,
                    bid_quantity: tick.bid_quantity,
                    ask_price: tick.ask_price,
                    ask_quantity: tick.ask_quantity,
                })
            }
            BinanceSwapsEvent::OpenInterest(oi) => {
                // TODO: Add OpenInterest variant to ExchangeEventData when needed
                // For now, this is a placeholder that creates a minimal trade event
                crate::models::exchange::ExchangeEventData::Trade(crate::models::exchange::TradeData {
                    event_time: oi.event_time,
                    transaction_time: oi.event_time,
                    trade_id: format!("oi_{}_{}", oi.instrument, oi.event_time.unix_timestamp()),
                    price: oi.open_interest,      // Using OI as price for now
                    quantity: oi.open_interest,   // Using OI as quantity for now
                    side: crate::MarketSide::Buy, // Placeholder
                    maker: false,
                })
            }
        }
    }

    /// Get the event time for this event.
    ///
    /// Returns the timestamp when this event was generated by the exchange.
    pub fn event_time(&self) -> time::UtcDateTime {
        match self {
            BinanceSwapsEvent::Trade(data) => data.event_time,
            BinanceSwapsEvent::AggTrade(data) => data.event_time,
            BinanceSwapsEvent::Book(data) => data.event_time,
            BinanceSwapsEvent::Tick(data) => data.event_time,
            BinanceSwapsEvent::OpenInterest(data) => data.event_time,
        }
    }

    /// Get the venue symbol for instrument lookup.
    ///
    /// Returns the trading pair symbol (e.g., "BTCUSDT") for this event.
    pub fn venue_symbol(&self) -> &str {
        match self {
            BinanceSwapsEvent::AggTrade(data) => &data.instrument,
            BinanceSwapsEvent::Tick(data) => &data.instrument,
            BinanceSwapsEvent::Trade(data) => &data.instrument,
            BinanceSwapsEvent::Book(data) => &data.instrument,
            BinanceSwapsEvent::OpenInterest(data) => &data.instrument,
        }
    }
}

impl fmt::Display for BinanceSwapsEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinanceSwapsEvent::AggTrade(data) => write!(f, "{}", data),
            BinanceSwapsEvent::Tick(data) => write!(f, "{}", data),
            BinanceSwapsEvent::Trade(data) => write!(f, "{:?}", data),
            BinanceSwapsEvent::Book(data) => write!(f, "{:?}", data),
            BinanceSwapsEvent::OpenInterest(data) => write!(
                f,
                "OpenInterest {} {} oi: {}",
                data.event_time
                    .format(TIMESTAMP_FORMAT)
                    .expect("Failed to format event time for binance open interest"),
                data.instrument,
                data.open_interest
            ),
        }
    }
}

/// Individual trade data from Binance.
///
/// Represents a single trade execution on Binance, including the trade type
/// and whether it was executed by a market maker or taker.
#[derive(Debug, Clone, Deserialize)]
pub struct BinanceSwapsTradeData {
    /// Event time (when the event was generated)
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    /// Transaction time (when the trade actually executed)
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: UtcDateTime,
    /// Trading pair symbol
    #[serde(rename = "s")]
    pub instrument: String,
    /// Trade ID
    #[serde(rename = "t")]
    pub trade_id: u64,
    /// Price
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Quantity
    #[serde(rename = "q")]
    pub quantity: Decimal,
    /// Trade type (MARKET, LIMIT, etc.)
    #[serde(rename = "X")]
    pub trade_type: String,
    /// Whether this trade was executed by a market maker (true) or taker (false)
    #[serde(rename = "m")]
    pub maker: bool,
}

/// Aggregated trade data from Binance.
///
/// Represents a collection of individual trades that have been aggregated
/// together for efficiency. Contains the range of trade IDs that were combined.
#[derive(Debug, Clone, Deserialize)]
pub struct BinanceSwapsAggTradeData {
    /// Event time
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    /// Transaction time of the last trade in this aggregate
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: UtcDateTime,
    /// Trading pair symbol
    #[serde(rename = "s")]
    pub instrument: String,
    /// Aggregated trade ID
    #[serde(rename = "a")]
    pub agg_trade_id: u64,
    /// First trade ID in this aggregate
    #[serde(rename = "f")]
    pub first_trade_id: u64,
    /// Last trade ID in this aggregate
    #[serde(rename = "l")]
    pub last_trade_id: u64,
    /// Price (weighted average for aggregated trades)
    #[serde(rename = "p")]
    pub price: Decimal,
    /// Total quantity
    #[serde(rename = "q")]
    pub quantity: Decimal,
    /// Whether the last trade was executed by a market maker
    #[serde(rename = "m")]
    pub maker: bool,
}

impl fmt::Display for BinanceSwapsAggTradeData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AggTrade {} {} id: {} price: {}, quantity: {}, maker: {}",
            self.event_time
                .format(TIMESTAMP_FORMAT)
                .expect("Failed to format event time for binance agg trade"),
            self.instrument,
            self.agg_trade_id,
            self.price,
            self.quantity,
            self.maker,
        )
    }
}

/// Order book update data from Binance.
///
/// Contains multiple price levels that were updated in a single order book event.
/// Used to maintain the current state of the order book.
#[derive(Debug, Clone, Deserialize)]
pub struct BinanceSwapsBookData {
    /// Event time
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    /// Transaction time
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: UtcDateTime,
    /// Trading pair symbol
    #[serde(rename = "s")]
    pub instrument: String,
    /// First update ID in this batch
    #[serde(rename = "U")]
    pub first_update_id: u64,
    /// Final update ID in this batch
    #[serde(rename = "u")]
    pub final_update_id: u64,
    /// Last final update ID (for synchronization)
    #[serde(rename = "pu")]
    pub last_final_update_id: u64,
    /// Updated bid (buy) price levels
    #[serde(rename = "b")]
    pub bids: Vec<BinanceSwapsBookUpdate>,
    /// Updated ask (sell) price levels
    #[serde(rename = "a")]
    pub asks: Vec<BinanceSwapsBookUpdate>,
}

/// Individual order book price level update from Binance.
///
/// Represents a single price level update in the order book,
/// containing the price and new quantity at that level.
#[derive(Debug, Clone, Deserialize)]
pub struct BinanceSwapsBookUpdate {
    /// Price level
    pub price: Decimal,
    /// Quantity at this price level
    pub quantity: Decimal,
}

/// Best bid/ask ticker data from Binance.
///
/// Contains the current best prices and available quantities
/// for immediate execution on both bid and ask sides.
#[derive(Debug, Clone, Deserialize)]
pub struct BinanceSwapsTickData {
    /// Event time
    #[serde(rename = "E", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
    /// Transaction time of the last trade
    #[serde(rename = "T", with = "custom_serde::timestamp")]
    pub transaction_time: UtcDateTime,
    /// Update ID
    #[serde(rename = "u")]
    pub update_id: u64,
    /// Trading pair symbol
    #[serde(rename = "s")]
    pub instrument: String,
    /// Best bid price
    #[serde(rename = "b")]
    pub bid_price: Decimal,
    /// Bid quantity
    #[serde(rename = "B")]
    pub bid_quantity: Decimal,
    /// Best ask price
    #[serde(rename = "a")]
    pub ask_price: Decimal,
    /// Ask quantity
    #[serde(rename = "A")]
    pub ask_quantity: Decimal,
}

/// Book ticker data from Binance Spot (without event timestamps).
///
/// Contains the current best prices and available quantities
/// for immediate execution on both bid and ask sides.
#[derive(Debug, Clone, Deserialize)]
pub struct BinanceSpotTickData {
    /// Update ID
    #[serde(rename = "u")]
    pub update_id: u64,
    /// Trading pair symbol
    #[serde(rename = "s")]
    pub instrument: String,
    /// Best bid price
    #[serde(rename = "b")]
    pub bid_price: Decimal,
    /// Bid quantity
    #[serde(rename = "B")]
    pub bid_quantity: Decimal,
    /// Best ask price
    #[serde(rename = "a")]
    pub ask_price: Decimal,
    /// Ask quantity
    #[serde(rename = "A")]
    pub ask_quantity: Decimal,
}

/// Open interest data from Binance.
///
/// Contains the current open interest for a futures contract,
/// including both the open interest value and timestamp.
#[derive(Debug, Clone, Deserialize)]
pub struct BinanceSwapsOpenInterestData {
    /// Trading pair symbol
    #[serde(rename = "symbol")]
    pub instrument: String,
    /// Open interest value
    #[serde(rename = "openInterest")]
    pub open_interest: Decimal,
    /// Event time
    #[serde(rename = "time", with = "custom_serde::timestamp")]
    pub event_time: UtcDateTime,
}

impl fmt::Display for BinanceSwapsTickData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tick {} {} id: {} bid: {} {} ask: {} {}",
            self.event_time
                .format(TIMESTAMP_FORMAT)
                .expect("Failed to format event time for binance tick"),
            self.instrument,
            self.update_id,
            self.bid_price,
            self.bid_quantity,
            self.ask_price,
            self.ask_quantity
        )
    }
}

impl fmt::Display for BinanceSpotTickData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Tick (Spot) {} id: {} bid: {} {} ask: {} {}",
            self.instrument, self.update_id, self.bid_price, self.bid_quantity, self.ask_price, self.ask_quantity
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binance_futures_trade() {
        let json_data = r#"{"stream":"btcusdt@trade","data":{"e":"trade","E":1676160600276,"T":1676160600269,"s":"BTCUSDT","t":3280342045,"p":"21845.10","q":"0.001","X":"MARKET","m":false}}"#;
        let _ = serde_json::from_str::<BinanceSwapsStreamEvent>(json_data).unwrap();
    }

    #[test]
    fn test_binance_futures_agg_trade() {
        let json_data = r#"{"stream":"gasusdt@aggTrade","data":{"e":"aggTrade","E":1698796800043,"a":3863267,"s":"GASUSDT","p":"6.279000","q":"141.2","f":15146241,"l":15146244,"T":1698796799890,"m":false}}"#;
        let _ = serde_json::from_str::<BinanceSwapsStreamEvent>(json_data).unwrap();
    }

    #[test]
    fn test_binance_spot_agg_trade() {
        let json_data = r#"{"stream":"btcusdt@aggTrade","data":{"e":"aggTrade","E":1735689659952,"s":"BTCUSDT","a":3358805182,"p":"93610.93000000","q":"0.01530000","f":4359938016,"l":4359938016,"T":1735689659952,"m":false,"M":true}}"#;
        let _ = serde_json::from_str::<BinanceSwapsStreamEvent>(json_data).unwrap();
    }

    #[test]
    fn test_binance_futures_book() {
        let json_data = r#"{"stream":"ethusdt@depth@0ms","data":{"e":"depthUpdate","E":1676024912112,"T":1676024912105,"s":"ETHUSDT","U":2487393816175,"u":2487393817473,"pu":2487393816141,"b":[["1543.11","18.655"],["1543.14","19.449"],["1543.17","0.004"],["1543.37","27.385"],["1543.82","87.566"],["1543.89","10.302"],["1543.93","0.000"],["1544.13","4.546"]],"a":[["1544.11","0.000"],["1544.13","0.000"],["1544.22","1.105"],["1544.44","42.495"],["1544.52","13.691"]]}}"#;
        let _ = serde_json::from_str::<BinanceSwapsStreamEvent>(json_data).unwrap();
    }

    #[test]
    fn test_binance_futures_ticker() {
        let json_data = r#"{"stream":"btcusdt@bookTicker","data":{"e":"bookTicker","u":2487455691211,"s":"BTCUSDT","b":"21840.40","B":"21.292","a":"21840.50","A":"11.169","T":1676026461537,"E":1676026461542}}"#;
        let _ = serde_json::from_str::<BinanceSwapsStreamEvent>(json_data).unwrap();
    }

    #[test]
    fn test_binance_spot_ticker() {
        // Spot ticker doesn't have stream wrapper and uses BinanceSpotTickData
        let json_data = r#"{"stream":"btcusdt@bookTicker","data":{"u":76861015827,"s":"BTCUSDT","b":"109257.64000000","B":"0.77163000","a":"109257.65000000","A":"6.27706000"}}"#;

        // Parse the wrapper manually to extract the data
        #[derive(Deserialize)]
        struct SpotTickerWrapper {
            stream: String,
            data: BinanceSpotTickData,
        }

        let wrapper = serde_json::from_str::<SpotTickerWrapper>(json_data).unwrap();
        assert_eq!(wrapper.data.instrument, "BTCUSDT");
        assert!(wrapper.data.bid_price > Decimal::ZERO);
        assert!(wrapper.data.ask_price > Decimal::ZERO);
    }

    #[test]
    fn test_binance_futures_open_interest() {
        let json_data = r#"{"stream":"btcusdt@openInterest","generated":true,"data":{"symbol":"BTCUSDT","openInterest":"91235.490","time":1735689598317}}"#;
        let _ = serde_json::from_str::<BinanceSwapsStreamEvent>(json_data).unwrap();
    }
}
