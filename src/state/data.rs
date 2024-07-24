use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    time::Duration,
};

use crate::{
    constants::TIMESTAMP_FORMAT,
    models::{Event, EventID, Instrument, Trade},
};
use parking_lot::RwLock;
use scc::{ebr::Guard, TreeIndex};
use time::OffsetDateTime;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, error, info};

#[derive(Default)]
pub struct StateData {
    events: TreeIndex<CompositeKey, Event>,
    pub subscribers: RwLock<HashMap<EventID, Sender<Event>>>,
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
    pub fn subscribe(&self, event_id: EventID) -> Receiver<Event> {
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
        // Add to state
        let mut key = CompositeKey::new(event.event_time());
        while self.events.insert_async(key.clone(), event.clone()).await.is_err() {
            key.increment();
        }
        debug!(
            "State added event: {}, now holds {} events",
            event.event_type(),
            self.events.len()
        );

        // Notify subscribers
        for (id, sender) in self.subscribers.read().iter() {
            if event.event_type() == id {
                if let Err(e) = sender.send(event.clone()) {
                    error!("Failed to send event: {}", e);
                }
            }
        }
    }

    pub fn list_instruments(&self) -> HashSet<Instrument> {
        let guard = Guard::new();
        self.events.iter(&guard).map(|(_, e)| e.instrument()).cloned().collect()
    }

    pub fn list_events<F>(&self, from: OffsetDateTime, window: Duration, predicate: F) -> Vec<Event>
    where
        F: Fn(&Event) -> Option<&Event>,
    {
        let from_adjusted = from - Duration::from_nanos(1);
        let till = from - (window);

        info!(
            "Getting data from: {} till: {}",
            from_adjusted.format(TIMESTAMP_FORMAT).expect("Unable to format timestamp"),
            till.format(TIMESTAMP_FORMAT).expect("Unable to format timestamp")
        );

        let from_key = CompositeKey::new_max(&from_adjusted);
        let end_key = CompositeKey::new(&till);

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

    pub fn list_trades<F>(&self, from: OffsetDateTime, window: Duration, predicate: F) -> Vec<Trade>
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
