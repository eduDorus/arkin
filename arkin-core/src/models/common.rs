use clap::ValueEnum;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use strum::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Type, Serialize, Deserialize, Default, ValueEnum)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "market_side", rename_all = "snake_case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketSide {
    #[default]
    Buy,
    Sell,
}

impl From<MarketSide> for f64 {
    fn from(side: MarketSide) -> f64 {
        match side {
            MarketSide::Buy => 1.0,
            MarketSide::Sell => -1.0,
        }
    }
}

impl From<MarketSide> for Decimal {
    fn from(side: MarketSide) -> Decimal {
        match side {
            MarketSide::Buy => Decimal::ONE,
            MarketSide::Sell => Decimal::NEGATIVE_ONE,
        }
    }
}

impl From<MarketSide> for i8 {
    fn from(side: MarketSide) -> i8 {
        match side {
            MarketSide::Buy => 1,
            MarketSide::Sell => -1,
        }
    }
}

impl From<i8> for MarketSide {
    fn from(side: i8) -> MarketSide {
        match side {
            1 => MarketSide::Buy,
            -1 => MarketSide::Sell,
            _ => panic!("Invalid market side: {}", side),
        }
    }
}
