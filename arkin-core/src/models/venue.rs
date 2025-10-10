use std::fmt;

use sqlx::prelude::Type;
use strum::{Display, EnumString};
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "venue_type", rename_all = "snake_case")]
pub enum VenueType {
    Cex,
    Dex,
    Otc,
    UserFunds,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, TypedBuilder)]
pub struct Venue {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub name: VenueName,
    pub venue_type: VenueType,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl fmt::Display for Venue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name.to_string().to_lowercase())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, EnumString, clap::ValueEnum)] // For Clap auto-parsing
#[strum(serialize_all = "snake_case")]
pub enum VenueName {
    Personal,
    BinanceSpot,
    BinanceUsdmFutures,
    BinanceCoinmFutures,
    BinanceOptions,
    OkxSpot,
    OkxSwap,
    OkxFutures,
    OkxOptions,
    BybitSpot,
    BybitDerivatives,
    BybitOptions,
    Deribit,
}

// Similarly for Channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, clap::ValueEnum)]
#[strum(serialize_all = "snake_case")]
pub enum Channel {
    OrderBook,
    Trades,
    AggTrades,
    Ticker,
    OpenInterest,
    FundingRate,
    LongShortRatio,
    Metrics,
    MarkPriceKlines,
    IndexPriceKlines,
}

impl Channel {
    pub fn channel_name_by_venue(&self, venue: VenueName) -> String {
        match (venue, self) {
            (VenueName::BinanceSpot, Channel::AggTrades) => "aggTrades".to_string(),
            (VenueName::BinanceSpot, Channel::Trades) => "trade".to_string(),
            (VenueName::BinanceUsdmFutures, Channel::AggTrades) => "aggTrades".to_string(),
            (VenueName::BinanceUsdmFutures, Channel::Trades) => "trade".to_string(),
            (VenueName::BinanceUsdmFutures, Channel::Metrics) => "metrics".to_string(),
            (VenueName::BinanceUsdmFutures, Channel::MarkPriceKlines) => "markPriceKlines".to_string(),
            (VenueName::BinanceUsdmFutures, Channel::IndexPriceKlines) => "indexPriceKlines".to_string(),
            (VenueName::BinanceCoinmFutures, Channel::AggTrades) => "aggTrades".to_string(),
            (VenueName::BinanceCoinmFutures, Channel::Trades) => "trade".to_string(),
            (VenueName::BinanceCoinmFutures, Channel::Metrics) => "metrics".to_string(),
            (VenueName::BinanceCoinmFutures, Channel::MarkPriceKlines) => "markPriceKlines".to_string(),
            (VenueName::BinanceCoinmFutures, Channel::IndexPriceKlines) => "indexPriceKlines".to_string(),
            // Default to lowercase channel name
            (_, channel) => channel.to_string().to_lowercase(),
        }
    }
    pub fn is_kline(&self) -> bool {
        matches!(self, Channel::MarkPriceKlines | Channel::IndexPriceKlines)
    }

    pub fn has_headers(&self, venue: VenueName) -> bool {
        match self {
            Self::AggTrades => !matches!(venue, VenueName::BinanceSpot),
            _ => true, // Metrics, klines default true
        }
    }
}
