use std::fmt;

use serde::{Deserialize, Serialize};
use sqlx::prelude::Type;
use strum::{Display, EnumIter, EnumString};
use time::UtcDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

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

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Type,
    Display,
    EnumString,
    EnumIter,
    clap::ValueEnum,
    Serialize,
    Deserialize,
)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "venue_name", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VenueName {
    Personal,
    Index,
    Binance,
    Okx,
    Bybit,
    Deribit,
    Coinbase,
    Hyperliquid,
}

#[derive(Debug, Display, Clone, PartialEq, Eq, Hash, Type, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "venue_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VenueType {
    Cex,
    Dex,
    Otc,
    UserFunds,
    Virtual,
}

// Similarly for Channel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, clap::ValueEnum, Deserialize)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    Ping,
    Instruments,
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
