use crate::{
    constants::TIMESTAMP_FORMAT,
    models::{Event, EventType, Instrument},
};
use parking_lot::RwLock;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, HashMap, HashSet},
    time::Duration,
};
use time::OffsetDateTime;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{error, info};

type EventMap = RwLock<HashMap<(Instrument, EventType), BTreeMap<CompositeKey, Event>>>;
type SubscriberMap = RwLock<HashMap<EventType, Sender<Event>>>;

#[derive(Default)]
pub struct DataStore {
    events: EventMap,
    subscribers: SubscriberMap,
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

impl DataStore {
    pub fn subscribe(&self, event_id: EventType) -> Receiver<Event> {
        info!("Subscribing to events: {}", event_id);
        if let Some(sender) = self.subscribers.read().get(&event_id) {
            info!("Found existing subscriber for frequency: {:?}", event_id);
            return sender.subscribe();
        }

        info!("Creating new subscriber for events: {}", event_id);
        let (sender, receiver) = broadcast::channel(1);
        self.subscribers.write().insert(event_id, sender);
        receiver
    }

    pub async fn add_event(&self, event: Event) {
        self.update_store(event.clone());
        self.update_subscribers(event);
    }

    fn update_store(&self, event: Event) {
        let key = (event.instrument().to_owned(), event.event_type().to_owned());

        // First acquire a read lock to check if the tree exists
        let mut events_guard = self.events.write();
        if let Some(tree) = events_guard.get_mut(&key) {
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
            events_guard.insert(key, new_tree);
        }
    }

    fn update_subscribers(&self, event: Event) {
        // Notify subscribers
        let subscribers_guard = self.subscribers.read();
        for (id, sender) in subscribers_guard.iter() {
            if event.event_type() == id {
                if let Err(e) = sender.send(event.clone()) {
                    error!("Failed to send event: {}", e);
                }
            }
        }
    }

    pub async fn list_instruments(&self) -> HashSet<Instrument> {
        self.events
            .read()
            .iter()
            .map(|((instrument, _), _)| instrument.to_owned())
            .collect()
    }

    pub async fn list_events(
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

        let events_guard = self.events.read();
        if let Some(tree) = events_guard.get(&(instrument.to_owned(), event_type)) {
            tree.range(end_key..=from_key).map(|(_, e)| e).cloned().collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    }
}
