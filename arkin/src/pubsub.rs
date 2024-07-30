use dashmap::DashMap;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{error, info};

use crate::models::{Event, EventType};

#[derive(Default)]
pub struct PubSub {
    subscribers: DashMap<EventType, Sender<Event>>,
}

impl PubSub {
    pub fn subscribe(&self, event_id: EventType) -> Receiver<Event> {
        info!("Subscribing to events: {}", event_id);
        if let Some(sender) = self.subscribers.get(&event_id) {
            info!("Found existing subscriber for frequency: {:?}", event_id);
            return sender.value().subscribe();
        }

        info!("Creating new subscriber for events: {}", event_id);
        let (sender, receiver) = broadcast::channel(1024);
        self.subscribers.insert(event_id, sender);
        receiver
    }

    pub fn publish(&self, event: Event) {
        // Notify subscribers
        for referance in self.subscribers.iter() {
            if event.event_type() == referance.key() {
                if let Err(e) = referance.value().send(event.clone()) {
                    error!("Failed to send event: {}", e);
                }
            }
        }
    }
}
