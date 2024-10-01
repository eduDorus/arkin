use std::{
    collections::{HashMap, HashSet},
    fmt,
};

use time::OffsetDateTime;

use crate::{
    models::{ExecutionOrder, Insight, Position, Signal, Tick, Trade},
    FeatureId, Instrument, Notional,
};

pub struct MarketSnapshot {
    timestamp: OffsetDateTime,
    ticks: HashMap<Instrument, Vec<Tick>>,
    trades: HashMap<Instrument, Vec<Trade>>,
}

impl MarketSnapshot {
    pub fn new(
        timestamp: OffsetDateTime,
        ticks: HashMap<Instrument, Vec<Tick>>,
        trades: HashMap<Instrument, Vec<Trade>>,
    ) -> Self {
        Self {
            timestamp,
            ticks,
            trades,
        }
    }

    pub fn timestamp(&self) -> OffsetDateTime {
        self.timestamp
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

    pub fn insights(&self) -> Vec<Insight> {
        let mut insights: Vec<Insight> = Vec::new();
        insights.extend(
            self.ticks
                .values()
                .flatten()
                .cloned()
                .map(|v| <Tick as Into<Vec<Insight>>>::into(v))
                .flatten(),
        );
        insights.extend(
            self.trades
                .values()
                .flatten()
                .cloned()
                .map(|v| <Trade as Into<Vec<Insight>>>::into(v))
                .flatten(),
        );
        insights
    }
}

impl fmt::Display for MarketSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MarketSnapshot: {}", self.timestamp)?;
        for (i, _) in &self.ticks {
            write!(f, "\n{}: {:?}", i, &self.last_tick(i).map(|f| f.mid_price()))?;
        }
        Ok(())
    }
}

pub struct PortfolioSnapshot {
    timestamp: OffsetDateTime,
    capital: Notional,
    positions: Vec<Position>,
}

impl PortfolioSnapshot {
    pub fn new(timestamp: OffsetDateTime, capital: Notional, positions: Vec<Position>) -> Self {
        Self {
            timestamp,
            capital,
            positions,
        }
    }

    pub fn timestamp(&self) -> OffsetDateTime {
        self.timestamp
    }

    pub fn capital(&self) -> Notional {
        self.capital
    }

    pub fn positions(&self) -> Vec<Position> {
        self.positions.clone()
    }
}

impl fmt::Display for PortfolioSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PositionSnapshot: {}", self.timestamp)?;
        Ok(())
    }
}

pub struct InsightsSnapshot {
    pub timestamp: OffsetDateTime,
    pub insights: Vec<Insight>,
}

impl InsightsSnapshot {
    pub fn new(timestamp: OffsetDateTime, insights: Vec<Insight>) -> Self {
        Self {
            timestamp,
            insights,
        }
    }

    pub fn timestamp(&self) -> OffsetDateTime {
        self.timestamp
    }

    pub fn insights(&self) -> Vec<Insight> {
        self.insights.clone()
    }

    pub fn instruments(&self) -> HashSet<Instrument> {
        self.insights.iter().map(|i| i.instrument.clone()).collect()
    }

    pub fn get_insight(&self, id: &FeatureId) -> Option<Insight> {
        self.insights.iter().find(|i| i.id() == id).cloned()
    }

    pub fn get_instrument_insight(&self, instrument: &Instrument, id: &FeatureId) -> Option<Insight> {
        self.insights
            .iter()
            .find(|insight| insight.instrument() == instrument && insight.id() == id)
            .cloned()
    }
}

pub struct StrategySnapshot {
    pub timestamp: OffsetDateTime,
    pub signals: Vec<Signal>,
}

impl StrategySnapshot {
    pub fn new(timestamp: OffsetDateTime, signals: Vec<Signal>) -> Self {
        Self { timestamp, signals }
    }

    pub fn timestamp(&self) -> OffsetDateTime {
        self.timestamp
    }

    pub fn signals(&self) -> Vec<Signal> {
        self.signals.clone()
    }
}

impl fmt::Display for StrategySnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SignalSnapshot: {}", self.timestamp)?;
        Ok(())
    }
}

pub struct AllocationSnapshot {
    pub timestamp: OffsetDateTime,
    pub orders: Vec<ExecutionOrder>,
}

impl AllocationSnapshot {
    pub fn new(timestamp: OffsetDateTime, orders: Vec<ExecutionOrder>) -> Self {
        Self { timestamp, orders }
    }

    pub fn timestamp(&self) -> OffsetDateTime {
        self.timestamp
    }

    pub fn orders(&self) -> Vec<ExecutionOrder> {
        self.orders.clone()
    }
}

impl fmt::Display for AllocationSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AllocationSnapshot: {}", self.timestamp)?;
        Ok(())
    }
}
