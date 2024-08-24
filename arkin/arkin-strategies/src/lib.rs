mod config;
mod crossover;
mod factory;
mod manager;

pub use config::*;
pub use crossover::CrossoverStrategy;
pub use manager::StrategyManager;
pub use manager::StrategyModule;

pub mod prelude {
    pub use crate::config::*;
    pub use crate::manager::*;
}
