use std::{collections::HashMap, fmt};

use time::OffsetDateTime;

use crate::{
    models::{Allocation, ExecutionOrder, Insight, Position, Signal, Tick, Trade},
    Instrument, StrategyId,
};

pub struct MarketSnapshot {
    event_time: OffsetDateTime,
    ticks: HashMap<Instrument, Vec<Tick>>,
    trades: HashMap<Instrument, Vec<Trade>>,
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

    pub fn last_tick(&self, instrument: &Instrument) -> Option<Tick> {
        self.ticks
            .get(instrument)
            .and_then(|ticks| ticks.last().map(|tick| tick.clone()))
    }

    pub fn last_trade(&self, instrument: &Instrument) -> Option<Trade> {
        self.trades
            .get(instrument)
            .and_then(|trades| trades.last().map(|trade| trade.clone()))
    }
}

impl fmt::Display for MarketSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MarketSnapshot: {}", self.event_time)?;
        for (i, _) in &self.ticks {
            write!(f, "\n{}: {:?}", i, &self.last_tick(i).map(|f| f.mid_price()))?;
        }
        Ok(())
    }
}

pub struct PortfolioSnapshot {
    pub timestamp: OffsetDateTime,
    pub positions: HashMap<(StrategyId, Instrument), Position>,
}

impl PortfolioSnapshot {
    pub fn new(timestamp: OffsetDateTime, positions: HashMap<(StrategyId, Instrument), Position>) -> Self {
        Self {
            timestamp,
            positions,
        }
    }
}

impl fmt::Display for PortfolioSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PositionSnapshot: {}", self.timestamp)?;
        Ok(())
    }
}

pub struct InsightsSnapshot {
    pub event_time: OffsetDateTime,
    pub insights: HashMap<Instrument, Insight>,
}

impl InsightsSnapshot {
    pub fn new(event_time: OffsetDateTime, features: HashMap<Instrument, Insight>) -> Self {
        Self {
            event_time,
            insights: features,
        }
    }
}

pub struct StrategySnapshot {
    pub event_time: OffsetDateTime,
    pub signals: HashMap<(StrategyId, Instrument), Signal>,
}

impl StrategySnapshot {
    pub fn new(event_time: OffsetDateTime, signals: HashMap<(StrategyId, Instrument), Signal>) -> Self {
        Self {
            event_time,
            signals,
        }
    }
}

impl fmt::Display for StrategySnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SignalSnapshot: {}", self.event_time)?;
        Ok(())
    }
}

pub struct AllocationSnapshot {
    pub event_time: OffsetDateTime,
    pub allocations: HashMap<(StrategyId, Instrument), Allocation>,
    pub orders: HashMap<(StrategyId, Instrument), ExecutionOrder>,
}

impl AllocationSnapshot {
    pub fn new(
        event_time: OffsetDateTime,
        allocations: HashMap<(StrategyId, Instrument), Allocation>,
        orders: HashMap<(StrategyId, Instrument), ExecutionOrder>,
    ) -> Self {
        Self {
            event_time,
            allocations,
            orders,
        }
    }
}

impl fmt::Display for AllocationSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AllocationSnapshot: {}", self.event_time)?;
        Ok(())
    }
}
