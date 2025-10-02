use anyhow::{anyhow, Result};
use arkin_core::prelude::*;

// Struct for declarative entries
#[derive(Debug, Clone, Copy)]
struct MappingEntry {
    exchange: Exchange,
    channel: Channel,
    tardis_id: &'static str,
    tardis_channel_str: &'static str,
}

// Getters: Linear scan (O(N) fine for small table)
pub fn get_tardis_exchange_id(exchange: Exchange) -> Result<&'static str> {
    MAPPINGS
        .iter()
        .find(|e| e.exchange == exchange)
        .map(|e| e.tardis_id)
        .ok_or(anyhow!("No mapping for exchange {}", exchange))
}

pub fn get_tardis_channel_str(exchange: Exchange, channel: Channel) -> Result<&'static str> {
    MAPPINGS
        .iter()
        .find(|e| e.exchange == exchange && e.channel == channel)
        .map(|e| e.tardis_channel_str)
        .ok_or(anyhow!("No channel {} for exchange {}", channel, exchange))
}

// Static table: Filter to Binance/OKX; expand as needed
const MAPPINGS: &[MappingEntry] = &[
    // BinanceSpot (verified: all match JSON)
    MappingEntry {
        exchange: Exchange::BinanceSpot,
        channel: Channel::Trades,
        tardis_id: "binance",
        tardis_channel_str: "trade",
    },
    MappingEntry {
        exchange: Exchange::BinanceSpot,
        channel: Channel::AggTrades,
        tardis_id: "binance",
        tardis_channel_str: "aggTrade",
    },
    MappingEntry {
        exchange: Exchange::BinanceSpot,
        channel: Channel::Ticker,
        tardis_id: "binance",
        tardis_channel_str: "bookTicker",
    },
    // BinanceUsdmFutures (verified: FundingRate via "markPrice")
    MappingEntry {
        exchange: Exchange::BinanceUsdmFutures,
        channel: Channel::Trades,
        tardis_id: "binance-futures",
        tardis_channel_str: "trade",
    },
    MappingEntry {
        exchange: Exchange::BinanceUsdmFutures,
        channel: Channel::AggTrades,
        tardis_id: "binance-futures",
        tardis_channel_str: "aggTrade",
    },
    MappingEntry {
        exchange: Exchange::BinanceUsdmFutures,
        channel: Channel::Ticker,
        tardis_id: "binance-futures",
        tardis_channel_str: "bookTicker",
    },
    MappingEntry {
        exchange: Exchange::BinanceUsdmFutures,
        channel: Channel::OpenInterest,
        tardis_id: "binance-futures",
        tardis_channel_str: "openInterest",
    },
    MappingEntry {
        exchange: Exchange::BinanceUsdmFutures,
        channel: Channel::LongShortRatio,
        tardis_id: "binance-futures",
        tardis_channel_str: "globalLongShortAccountRatio",
    },
    // BinanceCoinmFutures (COIN Futures)
    MappingEntry {
        exchange: Exchange::BinanceCoinmFutures,
        channel: Channel::Trades,
        tardis_id: "binance-delivery",
        tardis_channel_str: "trade",
    },
    MappingEntry {
        exchange: Exchange::BinanceCoinmFutures,
        channel: Channel::AggTrades,
        tardis_id: "binance-delivery",
        tardis_channel_str: "aggTrade",
    },
    MappingEntry {
        exchange: Exchange::BinanceCoinmFutures,
        channel: Channel::Ticker,
        tardis_id: "binance-delivery",
        tardis_channel_str: "bookTicker",
    },
    MappingEntry {
        exchange: Exchange::BinanceCoinmFutures,
        channel: Channel::OpenInterest,
        tardis_id: "binance-delivery",
        tardis_channel_str: "openInterest",
    },
    MappingEntry {
        exchange: Exchange::BinanceCoinmFutures,
        channel: Channel::LongShortRatio,
        tardis_id: "binance-delivery",
        tardis_channel_str: "globalLongShortAccountRatio",
    },
    // OkxSpot (verified: no AggTrades/FundingRate)
    MappingEntry {
        exchange: Exchange::OkxSpot,
        channel: Channel::AggTrades,
        tardis_id: "okex",
        tardis_channel_str: "trades",
    },
    MappingEntry {
        exchange: Exchange::OkxSpot,
        channel: Channel::Trades,
        tardis_id: "okex",
        tardis_channel_str: "trades-all",
    },
    MappingEntry {
        exchange: Exchange::OkxSpot,
        channel: Channel::Ticker,
        tardis_id: "okex",
        tardis_channel_str: "tickers",
    },
    // OkxSwap (verified: Trades as "trades-all")
    MappingEntry {
        exchange: Exchange::OkxSwap,
        channel: Channel::AggTrades,
        tardis_id: "okex-swap",
        tardis_channel_str: "trades",
    },
    MappingEntry {
        exchange: Exchange::OkxSwap,
        channel: Channel::Trades,
        tardis_id: "okex-swap",
        tardis_channel_str: "trades-all",
    },
    MappingEntry {
        exchange: Exchange::OkxSwap,
        channel: Channel::Ticker,
        tardis_id: "okex-swap",
        tardis_channel_str: "tickers",
    },
    MappingEntry {
        exchange: Exchange::OkxSwap,
        channel: Channel::OpenInterest,
        tardis_id: "okex-swap",
        tardis_channel_str: "open-interest",
    },
    // BybitSpot (fixed: "trade" not "trades"; OrderBook as "depth"; no AggTrades/OpenInterest/FundingRate)
    MappingEntry {
        exchange: Exchange::BybitSpot,
        channel: Channel::Trades,
        tardis_id: "bybit-spot",
        tardis_channel_str: "publicTrade", // was "trade" at api v3
    },
    MappingEntry {
        exchange: Exchange::BybitSpot,
        channel: Channel::Ticker,
        tardis_id: "bybit-spot",
        tardis_channel_str: "orderbook.1", // Use orderbook.1 for ticker data as it has bid/ask prices
    },
    // BybitDerivatives
    MappingEntry {
        exchange: Exchange::BybitDerivatives,
        channel: Channel::Trades,
        tardis_id: "bybit",
        tardis_channel_str: "publicTrade", // was "trade" at api v3
    },
    MappingEntry {
        exchange: Exchange::BybitDerivatives,
        channel: Channel::Ticker,
        tardis_id: "bybit",
        tardis_channel_str: "orderbook.1", // Use orderbook.1 for ticker data as it has bid/ask prices
    },
    MappingEntry {
        exchange: Exchange::BybitDerivatives,
        channel: Channel::OpenInterest,
        tardis_id: "bybit",
        tardis_channel_str: "tickers",
    },
    MappingEntry {
        exchange: Exchange::BybitDerivatives,
        channel: Channel::LongShortRatio,
        tardis_id: "bybit",
        tardis_channel_str: "long_short_ratio",
    },
];
