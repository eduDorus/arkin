use std::fmt;

use sqlx::prelude::Type;
use strum::Display;
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
    pub name: String,
    pub venue_type: VenueType,
    pub created: UtcDateTime,
    pub updated: UtcDateTime,
}

impl fmt::Display for Venue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name.to_lowercase())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, clap::ValueEnum)] // For Clap auto-parsing
#[strum(serialize_all = "snake_case")]
pub enum Exchange {
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
}
