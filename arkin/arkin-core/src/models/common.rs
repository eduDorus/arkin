use strum::Display;

#[derive(Display, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Buy,
    Sell,
}
