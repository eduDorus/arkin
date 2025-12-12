use anyhow::{anyhow, Result};
use arkin_core::prelude::*;

// Struct for declarative entries
#[derive(Debug, Clone, Copy)]
struct MappingEntry {
    venue: VenueName,
    instrument_type: InstrumentType,
    channel: Channel,
    tardis_id: &'static str,
    tardis_channel_str: &'static str,
}

// Getters: Linear scan (O(N) fine for small table)
pub fn get_tardis_exchange_id(exchange: VenueName) -> Result<&'static str> {
    MAPPINGS
        .iter()
        .find(|e| e.venue == exchange)
        .map(|e| e.tardis_id)
        .ok_or(anyhow!("No mapping for exchange {}", exchange))
}

pub fn get_tardis_channel_str(exchange: VenueName, channel: Channel) -> Result<&'static str> {
    MAPPINGS
        .iter()
        .find(|e| e.venue == exchange && e.channel == channel)
        .map(|e| e.tardis_channel_str)
        .ok_or(anyhow!("No channel {} for exchange {}", channel, exchange))
}

// Static table: Filter to Binance/OKX; expand as needed
const MAPPINGS: &[MappingEntry] = &[
    // BinanceSpot (verified: all match JSON)
    MappingEntry {
        venue: VenueName::Binance,
        instrument_type: InstrumentType::Spot,
        channel: Channel::Trades,
        tardis_id: "binance",
        tardis_channel_str: "trade",
    },
    MappingEntry {
        venue: VenueName::Binance,
        instrument_type: InstrumentType::Spot,
        channel: Channel::AggTrades,
        tardis_id: "binance",
        tardis_channel_str: "aggTrade",
    },
    MappingEntry {
        venue: VenueName::Binance,
        instrument_type: InstrumentType::Spot,
        channel: Channel::Ticker,
        tardis_id: "binance",
        tardis_channel_str: "bookTicker",
    },
    // BinanceUsdmFutures (verified: FundingRate via "markPrice")
    MappingEntry {
        venue: VenueName::Binance,
        instrument_type: InstrumentType::Perpetual,
        channel: Channel::Trades,
        tardis_id: "binance-futures",
        tardis_channel_str: "trade",
    },
    MappingEntry {
        venue: VenueName::Binance,
        instrument_type: InstrumentType::Perpetual,
        channel: Channel::AggTrades,
        tardis_id: "binance-futures",
        tardis_channel_str: "aggTrade",
    },
    MappingEntry {
        venue: VenueName::Binance,
        instrument_type: InstrumentType::Perpetual,
        channel: Channel::Ticker,
        tardis_id: "binance-futures",
        tardis_channel_str: "bookTicker",
    },
    MappingEntry {
        venue: VenueName::Binance,
        instrument_type: InstrumentType::Perpetual,
        channel: Channel::OpenInterest,
        tardis_id: "binance-futures",
        tardis_channel_str: "openInterest",
    },
    MappingEntry {
        venue: VenueName::Binance,
        instrument_type: InstrumentType::Perpetual,
        channel: Channel::LongShortRatio,
        tardis_id: "binance-futures",
        tardis_channel_str: "globalLongShortAccountRatio",
    },
    // BinanceCoinmFutures (COIN Futures)
    MappingEntry {
        venue: VenueName::Binance,
        instrument_type: InstrumentType::InversePerpetual,
        channel: Channel::Trades,
        tardis_id: "binance-delivery",
        tardis_channel_str: "trade",
    },
    MappingEntry {
        venue: VenueName::Binance,
        instrument_type: InstrumentType::InversePerpetual,
        channel: Channel::AggTrades,
        tardis_id: "binance-delivery",
        tardis_channel_str: "aggTrade",
    },
    MappingEntry {
        venue: VenueName::Binance,
        instrument_type: InstrumentType::InversePerpetual,
        channel: Channel::Ticker,
        tardis_id: "binance-delivery",
        tardis_channel_str: "bookTicker",
    },
    MappingEntry {
        venue: VenueName::Binance,
        instrument_type: InstrumentType::InversePerpetual,
        channel: Channel::OpenInterest,
        tardis_id: "binance-delivery",
        tardis_channel_str: "openInterest",
    },
    MappingEntry {
        venue: VenueName::Binance,
        instrument_type: InstrumentType::InversePerpetual,
        channel: Channel::LongShortRatio,
        tardis_id: "binance-delivery",
        tardis_channel_str: "globalLongShortAccountRatio",
    },
    // OkxSpot (verified: no AggTrades/FundingRate)
    MappingEntry {
        venue: VenueName::Okx,
        instrument_type: InstrumentType::Spot,
        channel: Channel::AggTrades,
        tardis_id: "okex",
        tardis_channel_str: "trades",
    },
    MappingEntry {
        venue: VenueName::Okx,
        instrument_type: InstrumentType::Spot,
        channel: Channel::Trades,
        tardis_id: "okex",
        tardis_channel_str: "trades-all",
    },
    MappingEntry {
        venue: VenueName::Okx,
        instrument_type: InstrumentType::Spot,
        channel: Channel::Ticker,
        tardis_id: "okex",
        tardis_channel_str: "tickers",
    },
    // // OkxSwap (verified: Trades as "trades-all")
    // MappingEntry {
    //     venue: VenueName::OkxSwap,
    //     channel: Channel::AggTrades,
    //     tardis_id: "okex-swap",
    //     tardis_channel_str: "trades",
    // },
    // MappingEntry {
    //     venue: VenueName::OkxSwap,
    //     channel: Channel::Trades,
    //     tardis_id: "okex-swap",
    //     tardis_channel_str: "trades-all",
    // },
    // MappingEntry {
    //     venue: VenueName::OkxSwap,
    //     channel: Channel::Ticker,
    //     tardis_id: "okex-swap",
    //     tardis_channel_str: "tickers",
    // },
    // MappingEntry {
    //     venue: VenueName::OkxSwap,
    //     channel: Channel::OpenInterest,
    //     tardis_id: "okex-swap",
    //     tardis_channel_str: "open-interest",
    // },
    // // BybitSpot (fixed: "trade" not "trades"; OrderBook as "depth"; no AggTrades/OpenInterest/FundingRate)
    // MappingEntry {
    //     venue: VenueName::BybitSpot,
    //     channel: Channel::Trades,
    //     tardis_id: "bybit-spot",
    //     tardis_channel_str: "publicTrade", // was "trade" at api v3
    // },
    // MappingEntry {
    //     venue: VenueName::BybitSpot,
    //     channel: Channel::Ticker,
    //     tardis_id: "bybit-spot",
    //     tardis_channel_str: "orderbook.1", // Use orderbook.1 for ticker data as it has bid/ask prices
    // },
    // // BybitDerivatives
    // MappingEntry {
    //     venue: VenueName::BybitDerivatives,
    //     channel: Channel::Trades,
    //     tardis_id: "bybit",
    //     tardis_channel_str: "publicTrade", // was "trade" at api v3
    // },
    // MappingEntry {
    //     venue: VenueName::BybitDerivatives,
    //     channel: Channel::Ticker,
    //     tardis_id: "bybit",
    //     tardis_channel_str: "orderbook.1", // Use orderbook.1 for ticker data as it has bid/ask prices
    // },
    // MappingEntry {
    //     venue: VenueName::BybitDerivatives,
    //     channel: Channel::OpenInterest,
    //     tardis_id: "bybit",
    //     tardis_channel_str: "tickers",
    // },
    // MappingEntry {
    //     venue: VenueName::BybitDerivatives,
    //     channel: Channel::LongShortRatio,
    //     tardis_id: "bybit",
    //     tardis_channel_str: "long_short_ratio",
    // },
];
