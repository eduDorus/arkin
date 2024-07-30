use crate::{
    constants::TIMESTAMP_FORMAT,
    models::{Event, EventType, Instrument},
};
use dashmap::DashMap;
use std::{
    collections::{BTreeMap, HashSet},
    time::Duration,
};
use time::OffsetDateTime;
use tracing::info;

#[derive(Clone, PartialOrd, Ord, Eq, PartialEq, Hash)]
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

#[derive(Default)]
pub struct State {
    events: DashMap<(Instrument, EventType), BTreeMap<CompositeKey, Event>>,
}

impl State {
    pub fn add_event(&self, event: Event) {
        let key = (event.instrument().to_owned(), event.event_type().to_owned());

        // First acquire a read lock to check if the tree exists
        if let Some(mut tree) = self.events.get_mut(&key) {
            // Add to existing tree
            let mut key = CompositeKey::new(event.event_time());
            while tree.get(&key).is_some() {
                key.increment();
            }
            tree.insert(key, event.to_owned());
        } else {
            // Tree doesn't exist, so create and insert a new one
            let mut new_tree = BTreeMap::new();
            let composite_key = CompositeKey::new(event.event_time());
            new_tree.insert(composite_key, event.to_owned());
            self.events.insert(key, new_tree);
        }
    }

    pub fn list_instruments(&self) -> HashSet<Instrument> {
        self.events.iter().map(|k| k.key().0.clone()).collect()
    }

    pub fn list_events(
        &self,
        instrument: &Instrument,
        event_type: EventType,
        from: OffsetDateTime,
        window: Duration,
    ) -> Vec<Event> {
        let from_adjusted = from - Duration::from_nanos(1);
        let till = from - (window);

        info!(
            "Getting {} for {} from: {} till: {}",
            event_type,
            instrument,
            from_adjusted.format(TIMESTAMP_FORMAT).expect("Unable to format timestamp"),
            till.format(TIMESTAMP_FORMAT).expect("Unable to format timestamp")
        );

        let from_key = CompositeKey::new_max(&from_adjusted);
        let end_key = CompositeKey::new(&till);

        if let Some(tree) = self.events.get(&(instrument.to_owned(), event_type)) {
            tree.range(end_key..=from_key).map(|(_, e)| e).cloned().collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    }
}
