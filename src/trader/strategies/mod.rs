use std::fmt;

use crossover::CrossoverStrategy;
use spreader::Spreader;

mod crossover;
pub mod errors;
mod factory;
mod spreader;

pub use factory::StrategyFactory;

#[trait_variant::make(Send)]
pub trait Strategy: Clone {
    async fn start(&self);
}

#[derive(Clone)]
pub enum StrategyType {
    WideQuoter(CrossoverStrategy),
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
