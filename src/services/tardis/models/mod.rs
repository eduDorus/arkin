#![allow(dead_code)]
use serde::Deserialize;

mod binance;

use binance::*;

// mod okex;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TardisModel {
    BinanceSwapsTrade(BinanceSwapsTrade),
    BinanceSwapsAggTrade(BinanceSwapsAggTrade),
    BinanceSwapsBook(BinanceSwapsBook),
    BinanceSwapsTicker(BinanceSwapsTicker),
}
