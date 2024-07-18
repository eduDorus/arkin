use std::{cmp::Ordering, collections::HashMap};

use crate::models::{Event, EventType, Instrument};
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
    instrument: Instrument,
    event_type: EventType,
    index: u64,
}

impl CompositeKey {
    pub fn new(timestamp: &OffsetDateTime, instrument: &Instrument, event_type: &EventType) -> Self {
        CompositeKey {
            timestamp: timestamp.to_owned(),
            instrument: instrument.to_owned(),
            event_type: event_type.to_owned(),
            index: 0,
        }
    }

    pub fn new_max(timestamp: &OffsetDateTime, instrument: &Instrument, event_type: &EventType) -> Self {
        CompositeKey {
            timestamp: timestamp.to_owned(),
            instrument: instrument.to_owned(),
            event_type: event_type.to_owned(),
            index: u64::MAX,
        }
    }

    pub fn increment(&mut self) {
        self.index += 1;
    }
}

// TODO: Implement the Ord and PartialOrd traits for Instrument and EventType
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
        let mut key = CompositeKey::new(event.event_time(), event.instrument(), event.event_type());
        while self.events.insert_async(key.clone(), event.clone()).await.is_err() {
            key.increment();
        }
    }

    pub fn list_events(
        &self,
        instruments: &[Instrument],
        event_types: &[EventType],
        from: OffsetDateTime,
        window: Duration,
    ) -> HashMap<(Instrument, EventType), Vec<Event>> {
        let end_time = from - window;

        info!("Getting data from: {} till: {}", from, end_time);

        let guard = Guard::new();
        let mut results = HashMap::new();

        for instrument in instruments {
            for event_type in event_types {
                let from_key = CompositeKey::new_max(&from, instrument, event_type);
                let end_key = CompositeKey::new(&end_time, instrument, event_type);

                let events = self
                    .events
                    .range(end_key..=from_key, &guard)
                    .map(|(_, e)| e.clone())
                    .collect::<Vec<_>>();

                results.insert((instrument.to_owned(), event_type.to_owned()), events);
            }
        }

        results
    }
}
