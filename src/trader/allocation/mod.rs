use core::fmt;

use flume::Sender;
use limited::LimitedAllocation;
use rust_decimal::Decimal;

use crate::models::Instrument;

pub mod errors;
mod factory;
mod limited;

pub use factory::AllocationFactory;

#[trait_variant::make(Send)]
pub trait Allocation: Clone {
    async fn start(&self, sender: Sender<AllocationEvent>);
}

#[derive(Clone)]
pub enum AllocationEvent {
    Signal(Signal),
    // Weighted(Vec<Weighted>),
}

impl fmt::Display for AllocationEvent {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AllocationEvent::Signal(e) => write!(f, "Limit: {}", e),
            // AllocationEvent::Weighted(e) => write!(f, "Market: {}", e),
        }
    }
}

#[derive(Clone)]
pub struct Signal {
    instrument: Instrument,
    signal: Decimal,
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} signal {}", self.instrument, self.signal)
    }
}

#[derive(Clone)]
pub struct Weighted {
    instrument: Instrument,
    weight: Decimal,
}

impl fmt::Display for Weighted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} signal {}", self.instrument, self.weight)
    }
}

#[derive(Clone)]
pub enum AllocationType {
    Limited(LimitedAllocation),
}

impl fmt::Display for AllocationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllocationType::Limited(_) => write!(f, "Limit"),
        }
    }
}
