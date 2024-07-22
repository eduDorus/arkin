use std::cmp::Ordering;

use crate::models::{Event, Trade};
use scc::{ebr::Guard, TreeIndex};
use time::{Duration, OffsetDateTime};
use tracing::info;

#[derive(Default)]
pub struct StateData {
    events: TreeIndex<CompositeKey, Event>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct CompositeKey {
    timestamp: OffsetDateTime,
    index: u64,
}

impl CompositeKey {
    pub fn new(timestamp: &OffsetDateTime) -> Self {
        CompositeKey {
            timestamp: timestamp.to_owned(),
            index: 0,
        }
    }

    pub fn new_max(timestamp: &OffsetDateTime) -> Self {
        CompositeKey {
            timestamp: timestamp.to_owned(),
            index: u64::MAX,
        }
    }

    pub fn increment(&mut self) {
        self.index += 1;
    }
}

impl Ord for CompositeKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.cmp(&other.timestamp).then_with(|| self.index.cmp(&other.index))
    }
}

impl PartialOrd for CompositeKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl StateData {
    pub async fn add_event(&self, event: Event) {
        let mut key = CompositeKey::new(event.event_time());
        while self.events.insert_async(key.clone(), event.clone()).await.is_err() {
            key.increment();
        }
        info!(
            "State added event: {}, now holds {} events",
            event.event_type(),
            self.events.len()
        );
    }

    pub fn list_events<F>(&self, from: OffsetDateTime, window: Duration, predicate: F) -> Vec<Event>
    where
        F: Fn(&Event) -> Option<&Event>,
    {
        let end_time = from - (window + Duration::nanoseconds(1));

        info!("Getting data from: {} till: {}", from, end_time);

        let from_key = CompositeKey::new_max(&from);
        let end_key = CompositeKey::new(&end_time);

        let guard = Guard::new();

        let events = self
            .events
            .range(end_key..=from_key, &guard)
            .map(|(_, e)| e)
            .filter_map(predicate)
            .cloned()
            .collect::<Vec<_>>();

        events
    }

    pub fn list_agg_trades<F>(&self, from: OffsetDateTime, window: Duration, predicate: F) -> Vec<Trade>
    where
        F: Fn(&Event) -> Option<&Event>,
    {
        self.list_events(from, window, predicate)
            .iter()
            .filter_map(|event| {
                if let Event::TradeUpdate(trade) = event {
                    Some(trade)
                } else {
                    None
                }
            })
            .cloned()
            .collect()
    }
}
