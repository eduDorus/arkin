use std::{collections::HashMap, fmt};

use derive_builder::Builder;
use time::OffsetDateTime;

use crate::{
    models::{Allocation, ExecutionOrder, Insight, Position, Signal, Tick, Trade},
    Instrument,
};

#[derive(Clone, Builder)]
pub struct Snapshot {
    pub event_time: OffsetDateTime,
    pub portfolio: PortfolioSnapshot,
    pub market: MarketSnapshot,
    #[builder(default)]
    pub trades: Vec<Trade>,
    #[builder(default)]
    pub insights: Vec<Insight>,
    #[builder(default)]
    pub signals: Vec<Signal>,
    #[builder(default)]
    pub allocations: Vec<Allocation>,
    #[builder(default)]
    pub orders: Vec<ExecutionOrder>,
}

impl Snapshot {
    pub fn add_insights(&mut self, features: Vec<Insight>) {
        self.insights = features;
    }

    pub fn add_signals(&mut self, signals: Vec<Signal>) {
        self.signals = signals;
    }

    pub fn add_allocations(&mut self, allocations: Vec<Allocation>) {
        self.allocations = allocations;
    }

    pub fn add_orders(&mut self, orders: Vec<ExecutionOrder>) {
        self.orders = orders;
    }
}

#[derive(Clone)]
pub struct PortfolioSnapshot {
    pub timestamp: OffsetDateTime,
    pub positions: Vec<Position>,
}

impl PortfolioSnapshot {
    pub fn new(timestamp: OffsetDateTime, positions: Vec<Position>) -> Self {
        Self {
            timestamp,
            positions,
        }
    }
}

impl fmt::Display for PortfolioSnapshot {
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
    pub insights: Vec<Insight>,
}

impl FeatureSnapshot {
    pub fn new(event_time: OffsetDateTime, features: Vec<Insight>) -> Self {
        Self {
            event_time,
            insights: features,
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

#[derive(Clone)]
pub struct MarketSnapshot {
    pub event_time: OffsetDateTime,
    pub ticks: HashMap<Instrument, Vec<Tick>>,
    pub trades: HashMap<Instrument, Vec<Trade>>,
}

impl MarketSnapshot {
    pub fn new(
        event_time: OffsetDateTime,
        ticks: HashMap<Instrument, Vec<Tick>>,
        trades: HashMap<Instrument, Vec<Trade>>,
    ) -> Self {
        Self {
            event_time,
            ticks,
            trades,
        }
    }
}

impl fmt::Display for MarketSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MarketSnapshot: {}", self.event_time)?;
        Ok(())
    }
}
