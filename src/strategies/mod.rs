use crossover::CrossoverStrategy;
use spreader::Spreader;
use std::fmt;

mod crossover;
mod errors;
mod factory;
mod spreader;

pub use factory::StrategyFactory;

#[trait_variant::make(Send)]
pub trait Strategy: Clone {
    async fn start(&self);
}

#[derive(Clone)]
pub enum StrategyType {
    Crossover(CrossoverStrategy),
    Spreader(Spreader),
}

impl Strategy for StrategyType {
    async fn start(&self) {
        match self {
            StrategyType::Spreader(s) => s.start().await,
            StrategyType::Crossover(s) => s.start().await,
        }
    }
}

impl fmt::Display for StrategyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StrategyType::Crossover(_) => write!(f, "crossover"),
            StrategyType::Spreader(_) => write!(f, "spreader"),
        }
    }
}
