use std::{
    collections::{BTreeMap, HashSet},
    time::Duration,
};

use arkin_core::prelude::*;
use dashmap::DashMap;
use time::OffsetDateTime;

#[derive(Default)]
pub struct MarketState {
    events: DashMap<(Instrument, EventType), BTreeMap<CompositeIndex, Event>>,
}

impl MarketState {
    pub fn insert(&self, event: Event) {
        let key = (event.instrument(), event.event_type());
        let mut composit_key = CompositeIndex::new(&event.timestamp());

        let mut entry = self.events.entry(key).or_default();
        while entry.get(&composit_key).is_some() {
            composit_key.increment();
        }
        entry.insert(composit_key, event);
    }

    pub fn list_instruments<T>(&self) -> HashSet<Instrument>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        let event_type = T::event_type();
        self.events
            .iter()
            .filter_map(|k| {
                if k.key().1 == event_type {
                    Some(k.key().0.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn list_entries_window<T>(
        &self,
        instrument: &Instrument,
        timestamp: OffsetDateTime,
        window: Duration,
    ) -> Vec<T>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        let event_type = T::event_type();
        let index = CompositeIndex::new_max(timestamp);
        let end_index = CompositeIndex::new(&(*timestamp - *window));

        self.events
            .get(&(instrument.clone(), event_type))
            .map(|set| {
                // Perform a range query up to the maximum key
                set.range(end_index..index)
                    .filter_map(|(_, entry)| entry.clone().try_into().ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    #[allow(dead_code)]
    pub fn last_entry<T>(&self, instrument: &Instrument, timestamp: OffsetDateTime) -> Option<T>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        let event_type = T::event_type();
        let index = CompositeIndex::new_max(timestamp);
        self.events
            .get(&(instrument.clone(), event_type))
            .and_then(|tree| tree.value().range(..index).next_back().map(|entry| entry.1.clone()))
            .and_then(|event| event.try_into().ok())
    }

    #[allow(dead_code)]
    pub fn list_entries_since_start<T>(&self, instrument: &Instrument, timestamp: OffsetDateTime) -> Vec<T>
    where
        T: TryFrom<Event, Error = ()> + EventTypeOf,
    {
        let event_type = T::event_type();
        let index = CompositeIndex::new_max(timestamp);

        self.events
            .get(&(instrument.clone(), event_type))
            .map(|set| {
                // Perform a range query up to the maximum key
                set.range(..index)
                    .filter_map(|(_, entry)| entry.clone().try_into().ok())
                    .collect()
            })
            .unwrap_or_default()
    }
}
