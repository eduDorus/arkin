use std::fmt;

use time::OffsetDateTime;

use crate::models::{Allocation, ExecutionOrder, Feature, Position, Signal, Tick};

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

impl fmt::Display for PositionSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PositionSnapshot: {}", self.timestamp)?;
        for position in &self.positions {
            write!(f, "\n{}", position)?;
        }
        Ok(())
    }
}

pub struct FeatureSnapshot {
    pub event_time: OffsetDateTime,
    pub metrics: Vec<Feature>,
}

impl FeatureSnapshot {
    pub fn new(event_time: OffsetDateTime, features: Vec<Feature>) -> Self {
        Self {
            event_time,
            metrics: features,
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

impl fmt::Display for SignalSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SignalSnapshot: {}", self.event_time)?;
        for signal in &self.signals {
            write!(f, "\n{}", signal)?;
        }
        Ok(())
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

impl fmt::Display for MarketSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MarketSnapshot: {}", self.event_time)?;
        for tick in &self.ticks {
            write!(f, "\n{}", tick)?;
        }
        Ok(())
    }
}
