use std::{
    collections::{BTreeMap, HashMap},
    fmt,
};

use time::OffsetDateTime;
use tracing::warn;

use crate::{
    models::{Allocation, ExecutionOrder, Insight, Position, Signal, Tick, Trade},
    utils::CompositeIndex,
    Instrument, Price, StrategyId,
};

pub struct Snapshot {
    pub event_time: OffsetDateTime,
    pub market: MarketSnapshot,
    pub portfolio: PortfolioSnapshot,
    pub insights: InsightsSnapshot,
    pub signals: SignalSnapshot,
    pub allocations: AllocationSnapshot,
}

impl Snapshot {
    pub fn add_insights(&mut self, insights: InsightsSnapshot) {
        if self.event_time != insights.event_time {
            warn!(
                "Event time mismatch: Snapshot event time: {}, Insights event time: {}",
                self.event_time, insights.event_time
            );
        }
        self.insights = insights;
    }

    pub fn add_signals(&mut self, signals: SignalSnapshot) {
        self.signals = signals;
    }

    pub fn add_allocations(&mut self, allocations: AllocationSnapshot) {
        self.allocations = allocations;
    }
}

pub struct MarketSnapshot {
    event_time: OffsetDateTime,
    ticks: HashMap<Instrument, BTreeMap<CompositeIndex, Tick>>,
    trades: HashMap<Instrument, BTreeMap<CompositeIndex, Trade>>,
}

impl MarketSnapshot {
    pub fn new(
        event_time: OffsetDateTime,
        ticks: HashMap<Instrument, BTreeMap<CompositeIndex, Tick>>,
        trades: HashMap<Instrument, BTreeMap<CompositeIndex, Trade>>,
    ) -> Self {
        Self {
            event_time,
            ticks,
            trades,
        }
    }

    pub fn last_mid_price(&self, instrument: &Instrument) -> Option<Price> {
        self.ticks
            .get(instrument)
            .and_then(|ticks| ticks.values().last())
            .map(|tick| tick.mid_price())
    }

    pub fn last_bid_price(&self, instrument: &Instrument) -> Option<Price> {
        self.ticks
            .get(instrument)
            .and_then(|ticks| ticks.values().last())
            .map(|tick| tick.bid_price())
    }

    pub fn last_ask_price(&self, instrument: &Instrument) -> Option<Price> {
        self.ticks
            .get(instrument)
            .and_then(|ticks| ticks.values().last())
            .map(|tick| tick.ask_price())
    }

    pub fn last_trade_price(&self, instrument: &Instrument) -> Option<Price> {
        self.trades
            .get(instrument)
            .and_then(|trades| trades.values().last())
            .map(|trade| trade.price)
    }
}

impl fmt::Display for MarketSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MarketSnapshot: {}", self.event_time)?;
        for (i, _) in &self.ticks {
            write!(f, "\n{}: {:?}", i, &self.last_mid_price(i))?;
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
    pub insights: Vec<Insight>,
}

impl InsightsSnapshot {
    pub fn new(event_time: OffsetDateTime, features: Vec<Insight>) -> Self {
        Self {
            event_time,
            insights: features,
        }
    }
}

pub struct SignalSnapshot {
    pub event_time: OffsetDateTime,
    pub signals: HashMap<(StrategyId, Instrument), Signal>,
}

impl SignalSnapshot {
    pub fn new(event_time: OffsetDateTime, signals: HashMap<(StrategyId, Instrument), Signal>) -> Self {
        Self {
            event_time,
            signals,
        }
    }
}

impl fmt::Display for SignalSnapshot {
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
