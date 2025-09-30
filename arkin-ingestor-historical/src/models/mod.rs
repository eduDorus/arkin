mod binance;
mod bybit;
mod okx;

pub use binance::*;
pub use bybit::*;
pub use okx::*;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RawMarketData {
    BinanceAggTrade(BinanceSwapsAggTrade),
    BinanceSwapsTicker(BinanceSwapsTick),
    OkxAggTrade(OkxRoot),
    BybitAggTrade(BybitRoot),
}
