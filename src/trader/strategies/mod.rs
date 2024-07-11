use std::fmt;

use spreader::Spreader;
use wide_quoter::WideQuoter;

pub mod errors;
mod factory;
mod spreader;
mod wide_quoter;

pub use factory::StrategyFactory;

#[trait_variant::make(Send)]
pub trait Strategy: Clone {
    async fn start(&self);
}

#[derive(Clone)]
pub enum StrategyType {
    WideQuoter(WideQuoter),
    Spreader(Spreader),
}

impl Strategy for StrategyType {
    async fn start(&self) {
        match self {
            StrategyType::Spreader(s) => s.start().await,
            StrategyType::WideQuoter(s) => s.start().await,
        }
    }
}

impl fmt::Display for StrategyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrategyType::WideQuoter(_) => write!(f, "WideQuoter"),
            StrategyType::Spreader(_) => write!(f, "Spreader"),
        }
    }
}
