use arkin_common::prelude::*;
use rust_decimal::prelude::*;

use crate::{config::EqualConfig, manager::AllocationModule};

pub struct EqualAllocation {
    _max_allocation: Decimal,
    _max_allocation_per_underlier: Decimal,
}

impl EqualAllocation {
    pub fn from_config(config: &EqualConfig) -> Self {
        EqualAllocation {
            _max_allocation: config.max_allocation,
            _max_allocation_per_underlier: config.max_allocation_per_underlier,
        }
    }
}

impl AllocationModule for EqualAllocation {
    fn calculate(&self, _snapshot: &Snapshot) -> Vec<Allocation> {
        vec![]
    }
}
