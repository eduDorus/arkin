use anyhow::{anyhow, Result};
use arkin_core::prelude::*;

// Struct for declarative entries
#[derive(Debug, Clone, Copy)]
struct MappingEntry {
    exchange: VenueName,
    channel: Channel,
    tardis_id: &'static str,
    tardis_channel_str: &'static str,
}

// Getters: Linear scan (O(N) fine for small table)
pub fn get_tardis_exchange_id(exchange: VenueName) -> Result<&'static str> {
    MAPPINGS
        .iter()
        .find(|e| e.exchange == exchange)
        .map(|e| e.tardis_id)
        .ok_or(anyhow!("No mapping for exchange {}", exchange))
}

pub fn get_tardis_channel_str(exchange: VenueName, channel: Channel) -> Result<&'static str> {
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
        exchange: VenueName::BinanceSpot,
        channel: Channel::Trades,
        tardis_id: "binance",
        tardis_channel_str: "trade",
    },
    MappingEntry {
        exchange: VenueName::BinanceSpot,
        channel: Channel::AggTrades,
        tardis_id: "binance",
        tardis_channel_str: "aggTrade",
    },
    MappingEntry {
        exchange: VenueName::BinanceSpot,
        channel: Channel::Ticker,
        tardis_id: "binance",
        tardis_channel_str: "bookTicker",
    },
    // BinanceUsdmFutures (verified: FundingRate via "markPrice")
    MappingEntry {
        exchange: VenueName::BinanceUsdmFutures,
        channel: Channel::Trades,
        tardis_id: "binance-futures",
        tardis_channel_str: "trade",
    },
    MappingEntry {
        exchange: VenueName::BinanceUsdmFutures,
        channel: Channel::AggTrades,
        tardis_id: "binance-futures",
        tardis_channel_str: "aggTrade",
    },
    MappingEntry {
        exchange: VenueName::BinanceUsdmFutures,
        channel: Channel::Ticker,
        tardis_id: "binance-futures",
        tardis_channel_str: "bookTicker",
    },
    MappingEntry {
        exchange: VenueName::BinanceUsdmFutures,
        channel: Channel::OpenInterest,
        tardis_id: "binance-futures",
        tardis_channel_str: "openInterest",
    },
    MappingEntry {
        exchange: VenueName::BinanceUsdmFutures,
        channel: Channel::LongShortRatio,
        tardis_id: "binance-futures",
        tardis_channel_str: "globalLongShortAccountRatio",
    },
    // BinanceCoinmFutures (COIN Futures)
    MappingEntry {
        exchange: VenueName::BinanceCoinmFutures,
        channel: Channel::Trades,
        tardis_id: "binance-delivery",
        tardis_channel_str: "trade",
    },
    MappingEntry {
        exchange: VenueName::BinanceCoinmFutures,
        channel: Channel::AggTrades,
        tardis_id: "binance-delivery",
        tardis_channel_str: "aggTrade",
    },
    MappingEntry {
        exchange: VenueName::BinanceCoinmFutures,
        channel: Channel::Ticker,
        tardis_id: "binance-delivery",
        tardis_channel_str: "bookTicker",
    },
    MappingEntry {
        exchange: VenueName::BinanceCoinmFutures,
        channel: Channel::OpenInterest,
        tardis_id: "binance-delivery",
        tardis_channel_str: "openInterest",
    },
    MappingEntry {
        exchange: VenueName::BinanceCoinmFutures,
        channel: Channel::LongShortRatio,
        tardis_id: "binance-delivery",
        tardis_channel_str: "globalLongShortAccountRatio",
    },
    // OkxSpot (verified: no AggTrades/FundingRate)
    MappingEntry {
        exchange: VenueName::OkxSpot,
        channel: Channel::AggTrades,
        tardis_id: "okex",
        tardis_channel_str: "trades",
    },
    MappingEntry {
        exchange: VenueName::OkxSpot,
        channel: Channel::Trades,
        tardis_id: "okex",
        tardis_channel_str: "trades-all",
    },
    MappingEntry {
        exchange: VenueName::OkxSpot,
        channel: Channel::Ticker,
        tardis_id: "okex",
        tardis_channel_str: "tickers",
    },
    // OkxSwap (verified: Trades as "trades-all")
    MappingEntry {
        exchange: VenueName::OkxSwap,
        channel: Channel::AggTrades,
        tardis_id: "okex-swap",
        tardis_channel_str: "trades",
    },
    MappingEntry {
        exchange: VenueName::OkxSwap,
        channel: Channel::Trades,
        tardis_id: "okex-swap",
        tardis_channel_str: "trades-all",
    },
    MappingEntry {
        exchange: VenueName::OkxSwap,
        channel: Channel::Ticker,
        tardis_id: "okex-swap",
        tardis_channel_str: "tickers",
    },
    MappingEntry {
        exchange: VenueName::OkxSwap,
        channel: Channel::OpenInterest,
        tardis_id: "okex-swap",
        tardis_channel_str: "open-interest",
    },
    // BybitSpot (fixed: "trade" not "trades"; OrderBook as "depth"; no AggTrades/OpenInterest/FundingRate)
    MappingEntry {
        exchange: VenueName::BybitSpot,
        channel: Channel::Trades,
        tardis_id: "bybit-spot",
        tardis_channel_str: "publicTrade", // was "trade" at api v3
    },
    MappingEntry {
        exchange: VenueName::BybitSpot,
        channel: Channel::Ticker,
        tardis_id: "bybit-spot",
        tardis_channel_str: "orderbook.1", // Use orderbook.1 for ticker data as it has bid/ask prices
    },
    // BybitDerivatives
    MappingEntry {
        exchange: VenueName::BybitDerivatives,
        channel: Channel::Trades,
        tardis_id: "bybit",
        tardis_channel_str: "publicTrade", // was "trade" at api v3
    },
    MappingEntry {
        exchange: VenueName::BybitDerivatives,
        channel: Channel::Ticker,
        tardis_id: "bybit",
        tardis_channel_str: "orderbook.1", // Use orderbook.1 for ticker data as it has bid/ask prices
    },
    MappingEntry {
        exchange: VenueName::BybitDerivatives,
        channel: Channel::OpenInterest,
        tardis_id: "bybit",
        tardis_channel_str: "tickers",
    },
    MappingEntry {
        exchange: VenueName::BybitDerivatives,
        channel: Channel::LongShortRatio,
        tardis_id: "bybit",
        tardis_channel_str: "long_short_ratio",
    },
];
