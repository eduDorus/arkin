pub mod errors;

mod allocation;
mod default;
mod factory;
mod strategies;

use core::fmt;

use default::DefaultTrader;
pub use factory::TraderFactory;

#[trait_variant::make(Send)]
pub trait Trader: Clone {
    async fn start(&self);
}

#[derive(Clone)]
pub enum TraderType {
    Default(DefaultTrader),
}

impl Trader for TraderType {
    async fn start(&self) {
        match self {
            TraderType::Default(t) => t.start().await,
        }
    }
}

impl fmt::Display for TraderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TraderType::Default(_) => write!(f, "Default"),
        }
    }
}
