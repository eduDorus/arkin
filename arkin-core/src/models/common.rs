use strum::Display;

#[derive(Display, Clone, Copy, PartialEq, Eq)]
pub enum MarketSide {
    Buy,
    Sell,
}
