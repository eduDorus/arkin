use crate::{
    constants::TIMESTAMP_FORMAT,
    models::{Event, EventType, Instrument},
    utils::CompositeKey,
};
use dashmap::DashMap;
use std::{
    collections::{BTreeMap, HashSet},
    time::Duration,
};
use time::OffsetDateTime;
use tracing::info;

#[derive(Default)]
pub struct State {
    events: DashMap<(Instrument, EventType), BTreeMap<CompositeKey, Event>>,
}

impl State {
    pub fn add_event(&self, event: Event) {
        let key = (event.instrument().clone(), event.event_type().clone());
        let mut composit_key = CompositeKey::new(event.event_time());

        let mut entry = self.events.entry(key).or_default();
        while entry.get(&composit_key).is_some() {
            composit_key.increment();
        }
        entry.insert(composit_key, event);
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
