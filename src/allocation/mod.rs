use std::fmt;

use async_trait::async_trait;
use limited::LimitedAllocation;

mod factory;
mod limited;

pub use factory::AllocationFactory;

#[async_trait]
#[allow(unused)]
pub trait Allocation: Clone {
    async fn start(&self);
}

#[derive(Clone)]
pub enum AllocationType {
    Limited(LimitedAllocation),
}

#[async_trait]
impl Allocation for AllocationType {
    async fn start(&self) {
        match self {
            AllocationType::Limited(l) => l.start().await,
        }
    }
}

impl fmt::Display for AllocationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllocationType::Limited(_) => write!(f, "Limit"),
        }
    }
}
