use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Exchange identifier
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum Exchange {
    Binance,
    Okx,
    Bybit,
    Coinbase,
}

impl std::fmt::Display for Exchange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Exchange::Binance => write!(f, "binance"),
            Exchange::Okx => write!(f, "okx"),
            Exchange::Bybit => write!(f, "bybit"),
            Exchange::Coinbase => write!(f, "coinbase"),
        }
    }
}

/// Market type (spot vs derivatives)
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum MarketType {
    Spot,
    Perpetual,        // USDT-margined perpetuals (Binance USDT-M, OKX USDT-SWAP, Bybit USDT, Coinbase)
    InversePerpetual, // Coin-margined perpetuals (Binance USDS-M, OKX USDC-SWAP, Bybit USDC)
}

impl std::fmt::Display for MarketType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MarketType::Spot => write!(f, "spot"),
            MarketType::Perpetual => write!(f, "perpetual"),
            MarketType::InversePerpetual => write!(f, "inverse_perpetual"),
        }
    }
}

/// Data stream type
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum StreamType {
    /// Aggregate trades (best available at each price level)
    AggregateTrades,
    /// Individual trades
    Trades,
    /// 24h ticker updates
    Ticker24h,
    /// Real-time ticker (price, volume, best bid/ask)
    TickerRealtime,
    /// Mark price (funding rate tied to this)
    MarkPrice,
    /// Index price (composition of spot prices)
    IndexPrice,
    /// Liquidation events
    Liquidations,
    /// Funding rate updates
    FundingRate,
    /// Open interest
    OpenInterest,
    /// Best bid/ask without full book
    BestBidAsk,
}

impl std::fmt::Display for StreamType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StreamType::AggregateTrades => write!(f, "aggregate_trades"),
            StreamType::Trades => write!(f, "trades"),
            StreamType::Ticker24h => write!(f, "ticker_24h"),
            StreamType::TickerRealtime => write!(f, "ticker_realtime"),
            StreamType::MarkPrice => write!(f, "mark_price"),
            StreamType::IndexPrice => write!(f, "index_price"),
            StreamType::Liquidations => write!(f, "liquidations"),
            StreamType::FundingRate => write!(f, "funding_rate"),
            StreamType::OpenInterest => write!(f, "open_interest"),
            StreamType::BestBidAsk => write!(f, "best_bid_ask"),
        }
    }
}

/// Configuration for a specific stream endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StreamConfig {
    /// Exchange this stream belongs to
    pub exchange: Exchange,
    /// Market type (spot, perpetual, etc)
    pub market_type: MarketType,
    /// Type of data this stream provides
    pub stream_type: StreamType,
    /// WebSocket URL for this stream
    pub url: String,
    /// Subscription message format (exchange-specific)
    pub subscription_message: String,
    /// Update frequency (milliseconds, if applicable)
    pub update_frequency_ms: Option<u32>,
    /// Description
    pub description: String,
    /// Parameters for this stream (can be symbol, contract, etc)
    pub params: HashMap<String, String>,
}

impl StreamConfig {
    pub fn new(
        exchange: Exchange,
        market_type: MarketType,
        stream_type: StreamType,
        url: String,
        subscription_message: String,
        description: String,
    ) -> Self {
        Self {
            exchange,
            market_type,
            stream_type,
            url,
            subscription_message,
            update_frequency_ms: None,
            description,
            params: HashMap::new(),
        }
    }

    pub fn with_frequency(mut self, freq_ms: u32) -> Self {
        self.update_frequency_ms = Some(freq_ms);
        self
    }

    pub fn with_param(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.params.insert(key.into(), value.into());
        self
    }
}

/// ============================================================================
/// SUBSCRIPTION MESSAGES (for WebSocket connections)
/// ============================================================================

/// Binance subscription format: {"method":"SUBSCRIBE","params":["btcusdt@aggTrade"],"id":1}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BinanceSubscription {
    pub method: String,
    pub params: Vec<String>,
    pub id: u32,
}

impl BinanceSubscription {
    pub fn new(streams: Vec<String>) -> Self {
        Self {
            method: "SUBSCRIBE".to_string(),
            params: streams,
            id: 1,
        }
    }
}

/// OKX subscription format: {"op":"subscribe","args":[{"channel":"trades","instId":"BTC-USDT"}]}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OkxSubscription {
    pub op: String,
    pub args: Vec<OkxSubscriptionArg>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OkxSubscriptionArg {
    pub channel: String,
    #[serde(rename = "instId")]
    pub inst_id: String,
}

impl OkxSubscription {
    pub fn new(args: Vec<OkxSubscriptionArg>) -> Self {
        Self {
            op: "subscribe".to_string(),
            args,
        }
    }
}

/// Bybit subscription format: {"op":"subscribe","args":["publicTrade.BTCUSDT"]}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BybitSubscription {
    pub op: String,
    pub args: Vec<String>,
}

impl BybitSubscription {
    pub fn new(streams: Vec<String>) -> Self {
        Self {
            op: "subscribe".to_string(),
            args: streams,
        }
    }
}

/// Coinbase subscription format: {"type":"subscribe","product_ids":["BTC-USD"],"channel":"market_trades"}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CoinbaseSubscription {
    #[serde(rename = "type")]
    pub subscription_type: String,
    pub product_ids: Vec<String>,
    pub channel: String,
}

impl CoinbaseSubscription {
    pub fn new(product_ids: Vec<String>, channel: String) -> Self {
        Self {
            subscription_type: "subscribe".to_string(),
            product_ids,
            channel,
        }
    }
}

/// ============================================================================
/// BINANCE CONFIGURATION
/// ============================================================================

pub mod binance {
    use super::*;

    const BINANCE_SPOT_URL: &str = "wss://stream.binance.com:9443/ws";
    const BINANCE_FUTURES_URL: &str = "wss://fstream.binance.com/ws";
    const BINANCE_USDS_FUTURES_URL: &str = "wss://fstream.binance.com/ws";

    /// Binance SPOT market streams
    pub fn spot_streams() -> Vec<StreamConfig> {
        vec![
            StreamConfig::new(
                Exchange::Binance,
                MarketType::Spot,
                StreamType::AggregateTrades,
                BINANCE_SPOT_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["<symbol>@aggTrade"],"id":1}"#.to_string(),
                "Binance SPOT: Aggregate trades (best bid/ask at each price)".to_string(),
            )
            .with_frequency(100),
            StreamConfig::new(
                Exchange::Binance,
                MarketType::Spot,
                StreamType::Trades,
                BINANCE_SPOT_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["<symbol>@trade"],"id":1}"#.to_string(),
                "Binance SPOT: Individual trades".to_string(),
            )
            .with_frequency(100),
            StreamConfig::new(
                Exchange::Binance,
                MarketType::Spot,
                StreamType::Ticker24h,
                BINANCE_SPOT_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["<symbol>@ticker_1h"],"id":1}"#.to_string(),
                "Binance SPOT: 24h ticker (hourly update)".to_string(),
            )
            .with_frequency(3600000),
            StreamConfig::new(
                Exchange::Binance,
                MarketType::Spot,
                StreamType::TickerRealtime,
                BINANCE_SPOT_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["<symbol>@ticker"],"id":1}"#.to_string(),
                "Binance SPOT: Real-time ticker (price, volume, best bid/ask)".to_string(),
            )
            .with_frequency(1000),
            StreamConfig::new(
                Exchange::Binance,
                MarketType::Spot,
                StreamType::BestBidAsk,
                BINANCE_SPOT_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["<symbol>@bookTicker"],"id":1}"#.to_string(),
                "Binance SPOT: Best bid/ask updates".to_string(),
            ),
        ]
    }

    /// Binance Perpetual USDT (Futures) market streams
    pub fn perpetual_usdt_streams() -> Vec<StreamConfig> {
        vec![
            StreamConfig::new(
                Exchange::Binance,
                MarketType::Perpetual,
                StreamType::AggregateTrades,
                BINANCE_FUTURES_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["!<symbol>@aggTrade"],"id":1}"#.to_string(),
                "Binance USDT Perpetual: Aggregate trades".to_string(),
            )
            .with_frequency(100),
            StreamConfig::new(
                Exchange::Binance,
                MarketType::Perpetual,
                StreamType::Trades,
                BINANCE_FUTURES_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["!<symbol>@trade"],"id":1}"#.to_string(),
                "Binance USDT Perpetual: Individual trades".to_string(),
            ),
            StreamConfig::new(
                Exchange::Binance,
                MarketType::Perpetual,
                StreamType::MarkPrice,
                BINANCE_FUTURES_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["!<symbol>@markPrice"],"id":1}"#.to_string(),
                "Binance USDT Perpetual: Mark price (funding rate source)".to_string(),
            )
            .with_frequency(1000),
            StreamConfig::new(
                Exchange::Binance,
                MarketType::Perpetual,
                StreamType::FundingRate,
                BINANCE_FUTURES_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["!<symbol>@fundingRate"],"id":1}"#.to_string(),
                "Binance USDT Perpetual: Funding rate updates".to_string(),
            ),
            StreamConfig::new(
                Exchange::Binance,
                MarketType::Perpetual,
                StreamType::Liquidations,
                BINANCE_FUTURES_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["forceOrder@stream"],"id":1}"#.to_string(),
                "Binance USDT Perpetual: Liquidation events (all symbols)".to_string(),
            ),
            StreamConfig::new(
                Exchange::Binance,
                MarketType::Perpetual,
                StreamType::OpenInterest,
                BINANCE_FUTURES_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["!<symbol>@openInterest"],"id":1}"#.to_string(),
                "Binance USDT Perpetual: Open interest updates".to_string(),
            ),
        ]
    }

    /// Binance Perpetual USDS (Coin Margined) market streams
    pub fn perpetual_usds_streams() -> Vec<StreamConfig> {
        vec![
            StreamConfig::new(
                Exchange::Binance,
                MarketType::InversePerpetual,
                StreamType::AggregateTrades,
                BINANCE_USDS_FUTURES_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["!<symbol>@aggTrade"],"id":1}"#.to_string(),
                "Binance USDS Perpetual: Aggregate trades".to_string(),
            )
            .with_frequency(100),
            StreamConfig::new(
                Exchange::Binance,
                MarketType::InversePerpetual,
                StreamType::MarkPrice,
                BINANCE_USDS_FUTURES_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["!<symbol>@markPrice"],"id":1}"#.to_string(),
                "Binance USDS Perpetual: Mark price".to_string(),
            )
            .with_frequency(1000),
            StreamConfig::new(
                Exchange::Binance,
                MarketType::InversePerpetual,
                StreamType::FundingRate,
                BINANCE_USDS_FUTURES_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["!<symbol>@fundingRate"],"id":1}"#.to_string(),
                "Binance USDS Perpetual: Funding rate updates".to_string(),
            ),
            StreamConfig::new(
                Exchange::Binance,
                MarketType::InversePerpetual,
                StreamType::Liquidations,
                BINANCE_USDS_FUTURES_URL.to_string(),
                r#"{"method":"SUBSCRIBE","params":["forceOrder@stream"],"id":1}"#.to_string(),
                "Binance USDS Perpetual: Liquidation events".to_string(),
            ),
        ]
    }
}

/// ============================================================================
/// OKX CONFIGURATION
/// ============================================================================

pub mod okx {
    use super::*;

    const OKX_SPOT_URL: &str = "wss://ws.okx.com:8443/ws/v5/public";
    const OKX_PERPETUAL_URL: &str = "wss://ws.okx.com:8443/ws/v5/public";
    #[allow(dead_code)]
    const OKX_PRIVATE_URL: &str = "wss://ws.okx.com:8443/ws/v5/private";

    /// OKX SPOT market streams
    pub fn spot_streams() -> Vec<StreamConfig> {
        vec![
            StreamConfig::new(
                Exchange::Okx,
                MarketType::Spot,
                StreamType::Trades,
                OKX_SPOT_URL.to_string(),
                r#"{"op":"subscribe","args":[{"channel":"trades","instId":"<instId>"}]}"#.to_string(),
                "OKX SPOT: Individual trades".to_string(),
            )
            .with_frequency(100),
            StreamConfig::new(
                Exchange::Okx,
                MarketType::Spot,
                StreamType::TickerRealtime,
                OKX_SPOT_URL.to_string(),
                r#"{"op":"subscribe","args":[{"channel":"ticker","instId":"<instId>"}]}"#.to_string(),
                "OKX SPOT: Real-time ticker (price, volume)".to_string(),
            )
            .with_frequency(100),
            StreamConfig::new(
                Exchange::Okx,
                MarketType::Spot,
                StreamType::BestBidAsk,
                OKX_SPOT_URL.to_string(),
                r#"{"op":"subscribe","args":[{"channel":"bbo-tbt","instId":"<instId>"}]}"#.to_string(),
                "OKX SPOT: Best bid/ask (tick by tick)".to_string(),
            ),
        ]
    }

    /// OKX Perpetual (Futures) market streams
    pub fn perpetual_usd_streams() -> Vec<StreamConfig> {
        vec![
            StreamConfig::new(
                Exchange::Okx,
                MarketType::Perpetual,
                StreamType::Trades,
                OKX_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":[{"channel":"trades","instId":"<instId>"}]}"#.to_string(),
                "OKX Perpetual: Individual trades".to_string(),
            )
            .with_frequency(100),
            StreamConfig::new(
                Exchange::Okx,
                MarketType::Perpetual,
                StreamType::TickerRealtime,
                OKX_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":[{"channel":"ticker","instId":"<instId>"}]}"#.to_string(),
                "OKX Perpetual: Real-time ticker".to_string(),
            )
            .with_frequency(100),
            StreamConfig::new(
                Exchange::Okx,
                MarketType::Perpetual,
                StreamType::MarkPrice,
                OKX_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":[{"channel":"mark-price","instId":"<instId>"}]}"#.to_string(),
                "OKX Perpetual: Mark price".to_string(),
            )
            .with_frequency(1000),
            StreamConfig::new(
                Exchange::Okx,
                MarketType::Perpetual,
                StreamType::IndexPrice,
                OKX_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":[{"channel":"index-tickers","instId":"<instId>"}]}"#.to_string(),
                "OKX Perpetual: Index price".to_string(),
            )
            .with_frequency(1000),
            StreamConfig::new(
                Exchange::Okx,
                MarketType::Perpetual,
                StreamType::FundingRate,
                OKX_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":[{"channel":"funding-rate","instId":"<instId>"}]}"#.to_string(),
                "OKX Perpetual: Funding rate updates".to_string(),
            ),
            StreamConfig::new(
                Exchange::Okx,
                MarketType::Perpetual,
                StreamType::OpenInterest,
                OKX_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":[{"channel":"open-interest","instId":"<instId>"}]}"#.to_string(),
                "OKX Perpetual: Open interest".to_string(),
            ),
        ]
    }
}

/// ============================================================================
/// BYBIT CONFIGURATION
/// ============================================================================

pub mod bybit {
    use super::*;

    const BYBIT_SPOT_URL: &str = "wss://stream.bybit.com/v5/public/spot";
    const BYBIT_PERPETUAL_URL: &str = "wss://stream.bybit.com/v5/public/linear";
    #[allow(dead_code)]
    const BYBIT_PERPETUAL_INVERSE_URL: &str = "wss://stream.bybit.com/v5/public/inverse";

    /// Bybit SPOT market streams
    pub fn spot_streams() -> Vec<StreamConfig> {
        vec![
            StreamConfig::new(
                Exchange::Bybit,
                MarketType::Spot,
                StreamType::Trades,
                BYBIT_SPOT_URL.to_string(),
                r#"{"op":"subscribe","args":["publicTrade.<symbol>"]}"#.to_string(),
                "Bybit SPOT: Individual trades".to_string(),
            )
            .with_frequency(100),
            StreamConfig::new(
                Exchange::Bybit,
                MarketType::Spot,
                StreamType::TickerRealtime,
                BYBIT_SPOT_URL.to_string(),
                r#"{"op":"subscribe","args":["tickers.<symbol>"]}"#.to_string(),
                "Bybit SPOT: Real-time ticker".to_string(),
            )
            .with_frequency(100),
            StreamConfig::new(
                Exchange::Bybit,
                MarketType::Spot,
                StreamType::BestBidAsk,
                BYBIT_SPOT_URL.to_string(),
                r#"{"op":"subscribe","args":["bookticker.<symbol>"]}"#.to_string(),
                "Bybit SPOT: Best bid/ask".to_string(),
            ),
        ]
    }

    /// Bybit Perpetual USDT (Linear) market streams
    pub fn perpetual_usdt_streams() -> Vec<StreamConfig> {
        vec![
            StreamConfig::new(
                Exchange::Bybit,
                MarketType::Perpetual,
                StreamType::Trades,
                BYBIT_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":["publicTrade.<symbol>"]}"#.to_string(),
                "Bybit USDT Perpetual: Individual trades".to_string(),
            )
            .with_frequency(100),
            StreamConfig::new(
                Exchange::Bybit,
                MarketType::Perpetual,
                StreamType::TickerRealtime,
                BYBIT_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":["tickers.<symbol>"]}"#.to_string(),
                "Bybit USDT Perpetual: Real-time ticker".to_string(),
            )
            .with_frequency(100),
            StreamConfig::new(
                Exchange::Bybit,
                MarketType::Perpetual,
                StreamType::MarkPrice,
                BYBIT_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":["markPrice.<symbol>"]}"#.to_string(),
                "Bybit USDT Perpetual: Mark price".to_string(),
            )
            .with_frequency(1000),
            StreamConfig::new(
                Exchange::Bybit,
                MarketType::Perpetual,
                StreamType::FundingRate,
                BYBIT_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":["funding.<symbol>"]}"#.to_string(),
                "Bybit USDT Perpetual: Funding rate updates".to_string(),
            ),
            StreamConfig::new(
                Exchange::Bybit,
                MarketType::Perpetual,
                StreamType::OpenInterest,
                BYBIT_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":["openInterest.<symbol>"]}"#.to_string(),
                "Bybit USDT Perpetual: Open interest".to_string(),
            ),
            StreamConfig::new(
                Exchange::Bybit,
                MarketType::Perpetual,
                StreamType::Liquidations,
                BYBIT_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":["liquidation.<symbol>"]}"#.to_string(),
                "Bybit USDT Perpetual: Liquidation events".to_string(),
            ),
        ]
    }

    /// Bybit Perpetual USDC market streams
    pub fn perpetual_usdc_streams() -> Vec<StreamConfig> {
        vec![
            StreamConfig::new(
                Exchange::Bybit,
                MarketType::InversePerpetual,
                StreamType::Trades,
                BYBIT_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":["publicTrade.<symbol>"]}"#.to_string(),
                "Bybit USDC Perpetual: Individual trades".to_string(),
            )
            .with_frequency(100),
            StreamConfig::new(
                Exchange::Bybit,
                MarketType::InversePerpetual,
                StreamType::MarkPrice,
                BYBIT_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":["markPrice.<symbol>"]}"#.to_string(),
                "Bybit USDC Perpetual: Mark price".to_string(),
            )
            .with_frequency(1000),
            StreamConfig::new(
                Exchange::Bybit,
                MarketType::InversePerpetual,
                StreamType::FundingRate,
                BYBIT_PERPETUAL_URL.to_string(),
                r#"{"op":"subscribe","args":["funding.<symbol>"]}"#.to_string(),
                "Bybit USDC Perpetual: Funding rate updates".to_string(),
            ),
        ]
    }
}

/// ============================================================================
/// COINBASE CONFIGURATION
/// ============================================================================

pub mod coinbase {
    use super::*;

    const COINBASE_ENDPOINT: &str = "wss://advanced-trade-ws.coinbase.com";

    /// Coinbase SPOT market streams
    pub fn spot_streams() -> Vec<StreamConfig> {
        vec![
            StreamConfig::new(
                Exchange::Coinbase,
                MarketType::Spot,
                StreamType::Trades,
                COINBASE_ENDPOINT.to_string(),
                r#"{"type":"subscribe","product_ids":["<product_id>"],"channel":"market_trades"}"#.to_string(),
                "Coinbase SPOT: Market trades (aggregated, ~250ms batches)".to_string(),
            )
            .with_frequency(250),
            StreamConfig::new(
                Exchange::Coinbase,
                MarketType::Spot,
                StreamType::TickerRealtime,
                COINBASE_ENDPOINT.to_string(),
                r#"{"type":"subscribe","product_ids":["<product_id>"],"channel":"ticker"}"#.to_string(),
                "Coinbase SPOT: Real-time ticker (updates on match)".to_string(),
            ),
            StreamConfig::new(
                Exchange::Coinbase,
                MarketType::Spot,
                StreamType::Ticker24h,
                COINBASE_ENDPOINT.to_string(),
                r#"{"type":"subscribe","product_ids":["<product_id>"],"channel":"candles"}"#.to_string(),
                "Coinbase SPOT: 5-minute candles (ticker + OHLCV)".to_string(),
            )
            .with_frequency(5000),
        ]
    }
}

/// ============================================================================
/// REGISTRY & UTILITIES
/// ============================================================================

/// Registry of all available streams
pub struct MarketRegistry {
    streams: HashMap<(Exchange, MarketType, StreamType), StreamConfig>,
}

impl MarketRegistry {
    pub fn new() -> Self {
        let mut streams = HashMap::new();

        // Register Binance streams
        for stream in binance::spot_streams() {
            let key = (stream.exchange, stream.market_type, stream.stream_type.clone());
            streams.insert(key, stream);
        }
        for stream in binance::perpetual_usdt_streams() {
            let key = (stream.exchange, stream.market_type, stream.stream_type.clone());
            streams.insert(key, stream);
        }
        for stream in binance::perpetual_usds_streams() {
            let key = (stream.exchange, stream.market_type, stream.stream_type.clone());
            streams.insert(key, stream);
        }

        // Register OKX streams
        for stream in okx::spot_streams() {
            let key = (stream.exchange, stream.market_type, stream.stream_type.clone());
            streams.insert(key, stream);
        }
        for stream in okx::perpetual_usd_streams() {
            let key = (stream.exchange, stream.market_type, stream.stream_type.clone());
            streams.insert(key, stream);
        }

        // Register Bybit streams
        for stream in bybit::spot_streams() {
            let key = (stream.exchange, stream.market_type, stream.stream_type.clone());
            streams.insert(key, stream);
        }
        for stream in bybit::perpetual_usdt_streams() {
            let key = (stream.exchange, stream.market_type, stream.stream_type.clone());
            streams.insert(key, stream);
        }
        for stream in bybit::perpetual_usdc_streams() {
            let key = (stream.exchange, stream.market_type, stream.stream_type.clone());
            streams.insert(key, stream);
        }

        // Register Coinbase streams
        for stream in coinbase::spot_streams() {
            let key = (stream.exchange, stream.market_type, stream.stream_type.clone());
            streams.insert(key, stream);
        }

        Self { streams }
    }

    pub fn get_stream(
        &self,
        exchange: Exchange,
        market_type: MarketType,
        stream_type: StreamType,
    ) -> Option<StreamConfig> {
        self.streams.get(&(exchange, market_type, stream_type)).cloned()
    }

    pub fn get_streams_for_exchange(&self, exchange: Exchange) -> Vec<StreamConfig> {
        self.streams.values().filter(|s| s.exchange == exchange).cloned().collect()
    }

    pub fn get_streams_for_market_type(&self, market_type: MarketType) -> Vec<StreamConfig> {
        self.streams
            .values()
            .filter(|s| s.market_type == market_type)
            .cloned()
            .collect()
    }

    pub fn list_all(&self) -> Vec<StreamConfig> {
        self.streams.values().cloned().collect()
    }
}

impl Default for MarketRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_loads_all_streams() {
        let registry = MarketRegistry::new();
        let all_streams = registry.list_all();
        assert!(!all_streams.is_empty(), "Registry should contain streams");
    }

    #[test]
    fn test_get_binance_perpetual_usdt_aggtrades() {
        let registry = MarketRegistry::new();
        let stream = registry.get_stream(Exchange::Binance, MarketType::Perpetual, StreamType::AggregateTrades);
        assert!(stream.is_some());
        let config = stream.unwrap();
        assert_eq!(config.exchange, Exchange::Binance);
        assert_eq!(config.market_type, MarketType::Perpetual);
        assert_eq!(config.stream_type, StreamType::AggregateTrades);
    }

    #[test]
    fn test_get_streams_for_exchange() {
        let registry = MarketRegistry::new();
        let binance_streams = registry.get_streams_for_exchange(Exchange::Binance);
        assert!(binance_streams.len() > 5, "Should have multiple Binance streams");
    }

    #[test]
    fn test_okx_perpetual_funding_rate() {
        let registry = MarketRegistry::new();
        let stream = registry.get_stream(Exchange::Okx, MarketType::Perpetual, StreamType::FundingRate);
        assert!(stream.is_some());
    }

    #[test]
    fn test_bybit_spot_trades() {
        let registry = MarketRegistry::new();
        let stream = registry.get_stream(Exchange::Bybit, MarketType::Spot, StreamType::Trades);
        assert!(stream.is_some());
    }

    #[test]
    fn test_coinbase_ticker() {
        let registry = MarketRegistry::new();
        let stream = registry.get_stream(Exchange::Coinbase, MarketType::Spot, StreamType::TickerRealtime);
        assert!(stream.is_some());
    }
}
