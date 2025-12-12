mod monolith;
mod redis;

pub use monolith::*;
pub use redis::*;
use strum::IntoEnumIterator;

use crate::EventType;

pub enum EventFilter {
    All,
    None,
    AllWithoutMarket,
    Persistable,
    PersistableSimulation,
    InsightUpdates,
    Events(Vec<EventType>),
}

impl EventFilter {
    pub fn event_types(&self) -> Vec<EventType> {
        match self {
            EventFilter::All => EventType::iter().collect(),
            EventFilter::None => vec![],
            EventFilter::AllWithoutMarket => EventType::iter().filter(|et| !et.is_market_data()).collect(),
            EventFilter::Persistable => EventType::iter().filter(|et| et.is_persistable()).collect(),
            EventFilter::PersistableSimulation => EventType::iter().filter(|et| et.is_simulation()).collect(),
            EventFilter::InsightUpdates => EventType::iter().filter(|et| et.is_insight()).collect(),
            EventFilter::Events(events) => events.clone(),
        }
    }
}
