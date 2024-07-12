use core::fmt;

use backtest::BacktestExecution;
use binance::BinanceExecution;

use crate::models::{Instrument, Price, Quantity};

mod backtest;
mod binance;
pub mod errors;
mod factory;

pub use factory::ExecutionFactory;

#[trait_variant::make(Send)]
pub trait Execution: Clone {
    async fn start(&self);
}

#[derive(Clone)]
pub enum ExecutionType {
    Binance(BinanceExecution),
    Backtest(BacktestExecution),
}

impl Execution for ExecutionType {
    async fn start(&self) {
        match self {
            ExecutionType::Binance(exec) => exec.start().await,
            ExecutionType::Backtest(exec) => exec.start().await,
        }
    }
}

impl fmt::Display for ExecutionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutionType::Binance(_) => write!(f, "Binance"),
            ExecutionType::Backtest(_) => write!(f, "Backtest"),
        }
    }
}

#[derive(Clone)]
pub enum ExecutionEvent {
    Limit(Limit),
    Market(Market),
}

impl fmt::Display for ExecutionEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExecutionEvent::Limit(e) => write!(f, "Limit: {}", e),
            ExecutionEvent::Market(e) => write!(f, "Market: {}", e),
        }
    }
}

#[derive(Clone)]
pub struct Limit {
    instrument: Instrument,
    price: Price,
    quantity: Quantity,
}

impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at {} quantity {}", self.instrument, self.price, self.quantity)
    }
}

#[derive(Clone)]
pub struct Market {
    instrument: Instrument,
    quantity: Quantity,
}

impl fmt::Display for Market {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} quantity {}", self.instrument, self.quantity)
    }
}
