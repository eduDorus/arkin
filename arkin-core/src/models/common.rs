use rust_decimal::Decimal;
use sqlx::Type;
use strum::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Hash, Type)]
#[strum(serialize_all = "snake_case")]
#[sqlx(type_name = "market_side", rename_all = "snake_case")]
pub enum MarketSide {
    Buy,
    Sell,
}

impl From<MarketSide> for Decimal {
    fn from(side: MarketSide) -> Decimal {
        match side {
            MarketSide::Buy => Decimal::from(1),
            MarketSide::Sell => Decimal::from(-1),
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
