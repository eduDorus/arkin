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
        channel: Channel::OrderBook,
        tardis_id: "binance",
        tardis_channel_str: "depth",
    },
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
        channel: Channel::OrderBook,
        tardis_id: "binance-futures",
        tardis_channel_str: "depth",
    },
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
        channel: Channel::FundingRate,
        tardis_id: "binance-futures",
        tardis_channel_str: "markPrice",
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
        channel: Channel::OrderBook,
        tardis_id: "binance-delivery",
        tardis_channel_str: "depth",
    },
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
        channel: Channel::FundingRate,
        tardis_id: "binance-delivery",
        tardis_channel_str: "markPrice",
    },
    MappingEntry {
        exchange: Exchange::BinanceCoinmFutures,
        channel: Channel::LongShortRatio,
        tardis_id: "binance-delivery",
        tardis_channel_str: "globalLongShortAccountRatio",
    },
    // BinanceOptions (verified: European; no AggTrades)
    MappingEntry {
        exchange: Exchange::BinanceOptions,
        channel: Channel::OrderBook,
        tardis_id: "binance-european-options",
        tardis_channel_str: "depth100",
    },
    MappingEntry {
        exchange: Exchange::BinanceOptions,
        channel: Channel::Trades,
        tardis_id: "binance-european-options",
        tardis_channel_str: "trade",
    },
    MappingEntry {
        exchange: Exchange::BinanceOptions,
        channel: Channel::Ticker,
        tardis_id: "binance-european-options",
        tardis_channel_str: "ticker",
    },
    MappingEntry {
        exchange: Exchange::BinanceOptions,
        channel: Channel::OpenInterest,
        tardis_id: "binance-european-options",
        tardis_channel_str: "openInterest",
    },
    // OkxSpot (verified: no AggTrades/FundingRate)
    MappingEntry {
        exchange: Exchange::OkxSpot,
        channel: Channel::OrderBook,
        tardis_id: "okex",
        tardis_channel_str: "books",
    },
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
        channel: Channel::OrderBook,
        tardis_id: "okex-swap",
        tardis_channel_str: "books",
    },
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
    MappingEntry {
        exchange: Exchange::OkxSwap,
        channel: Channel::FundingRate,
        tardis_id: "okex-swap",
        tardis_channel_str: "funding-rate",
    },
    // BybitSpot (fixed: "trade" not "trades"; OrderBook as "depth"; no AggTrades/OpenInterest/FundingRate)
    MappingEntry {
        exchange: Exchange::BybitSpot,
        channel: Channel::OrderBook,
        tardis_id: "bybit-spot",
        tardis_channel_str: "depth",
    },
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
        tardis_channel_str: "bookTicker",
    },
    // BybitDerivatives (fixed: OrderBook as "orderBook_200"; Trades as "trade"; Ticker as "tickers"; OI/Funding in "tickers"; no AggTrades)
    MappingEntry {
        exchange: Exchange::BybitDerivatives,
        channel: Channel::OrderBook,
        tardis_id: "bybit",
        tardis_channel_str: "orderBook_200",
    },
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
        tardis_channel_str: "tickers",
    },
    MappingEntry {
        exchange: Exchange::BybitDerivatives,
        channel: Channel::OpenInterest,
        tardis_id: "bybit",
        tardis_channel_str: "tickers",
    },
    MappingEntry {
        exchange: Exchange::BybitDerivatives,
        channel: Channel::FundingRate,
        tardis_id: "bybit",
        tardis_channel_str: "tickers",
    },
    MappingEntry {
        exchange: Exchange::BybitDerivatives,
        channel: Channel::LongShortRatio,
        tardis_id: "bybit",
        tardis_channel_str: "long_short_ratio",
    },
    // BybitOptions (verified: no AggTrades; OI in "tickers")
    MappingEntry {
        exchange: Exchange::BybitOptions,
        channel: Channel::OrderBook,
        tardis_id: "bybit-options",
        tardis_channel_str: "orderbook.100",
    },
    MappingEntry {
        exchange: Exchange::BybitOptions,
        channel: Channel::Trades,
        tardis_id: "bybit-options",
        tardis_channel_str: "publicTrade",
    },
    MappingEntry {
        exchange: Exchange::BybitOptions,
        channel: Channel::Ticker,
        tardis_id: "bybit-options",
        tardis_channel_str: "tickers",
    },
    MappingEntry {
        exchange: Exchange::BybitOptions,
        channel: Channel::OpenInterest,
        tardis_id: "bybit-options",
        tardis_channel_str: "tickers",
    },
    // Deribit (new: no AggTrades; FundingRate via "perpetual"; OI in "ticker")
    MappingEntry {
        exchange: Exchange::Deribit,
        channel: Channel::OrderBook,
        tardis_id: "deribit",
        tardis_channel_str: "book",
    },
    MappingEntry {
        exchange: Exchange::Deribit,
        channel: Channel::Trades,
        tardis_id: "deribit",
        tardis_channel_str: "trades",
    },
    MappingEntry {
        exchange: Exchange::Deribit,
        channel: Channel::Ticker,
        tardis_id: "deribit",
        tardis_channel_str: "ticker",
    },
    MappingEntry {
        exchange: Exchange::Deribit,
        channel: Channel::OpenInterest,
        tardis_id: "deribit",
        tardis_channel_str: "ticker",
    },
    MappingEntry {
        exchange: Exchange::Deribit,
        channel: Channel::FundingRate,
        tardis_id: "deribit",
        tardis_channel_str: "perpetual",
    },
];
