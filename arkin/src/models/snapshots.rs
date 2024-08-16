use time::OffsetDateTime;

use super::{Allocation, ExecutionOrder, FeatureEvent, Position, Signal, Tick};

pub struct PositionSnapshot {
    pub timestamp: OffsetDateTime,
    pub positions: Vec<Position>,
}

impl PositionSnapshot {
    pub fn new(timestamp: OffsetDateTime, positions: Vec<Position>) -> Self {
        Self {
            timestamp,
            positions,
        }
    }
}

pub struct FeatureSnapshot {
    pub event_time: OffsetDateTime,
    pub features: Vec<FeatureEvent>,
}

impl FeatureSnapshot {
    pub fn new(event_time: OffsetDateTime, features: Vec<FeatureEvent>) -> Self {
        Self {
            event_time,
            features,
        }
    }
}

pub struct SignalSnapshot {
    pub event_time: OffsetDateTime,
    pub signals: Vec<Signal>,
}

impl SignalSnapshot {
    pub fn new(event_time: OffsetDateTime, signals: Vec<Signal>) -> Self {
        Self {
            event_time,
            signals,
        }
    }
}

pub struct AllocationSnapshot {
    pub event_time: OffsetDateTime,
    pub allocations: Vec<Allocation>,
    pub orders: Vec<ExecutionOrder>,
}

impl AllocationSnapshot {
    pub fn new(event_time: OffsetDateTime, allocations: Vec<Allocation>, orders: Vec<ExecutionOrder>) -> Self {
        Self {
            event_time,
            allocations,
            orders,
        }
    }
}

pub struct MarketSnapshot {
    pub event_time: OffsetDateTime,
    pub ticks: Vec<Tick>,
}

impl MarketSnapshot {
    pub fn new(event_time: OffsetDateTime, ticks: Vec<Tick>) -> Self {
        Self { event_time, ticks }
    }
}
