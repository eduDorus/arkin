use arkin_core::prelude::*;
use serde::Deserialize;

// Re-export exchange models from core
pub use arkin_core::models::exchange::*;

/// Unified enum for all raw market data from different exchanges
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RawMarketData {
    BinanceAggTrade(BinanceSwapsAggTradeData),
    BinanceTick(BinanceSwapsTickData),
    OkxAggTrade(OkxTrade),
    BybitAggTrade(BybitTrade),
}
