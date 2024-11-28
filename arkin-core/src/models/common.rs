use rust_decimal::Decimal;
use strum::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
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
