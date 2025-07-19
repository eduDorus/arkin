use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use time::UtcDateTime;
use tracing::{debug, instrument};

use arkin_core::prelude::*;

pub struct Audit {
    identifier: String,
    event_log: DashMap<UtcDateTime, Event>,
}

impl Audit {
    pub fn new(indentifier: &str) -> Arc<Self> {
        Self {
            identifier: indentifier.to_owned(),
            event_log: DashMap::new(),
        }
        .into()
    }

    pub fn event_log(&self) -> Vec<Event> {
        let mut events = self.event_log.iter().map(|e| e.value().clone()).collect::<Vec<_>>();
        events.sort_unstable();
        events
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn add_event_to_log(&self, event: Event) {
        self.event_log.insert(event.timestamp(), event.to_owned());
        debug!(target: "audit", "add new event {} to log ({} logs)", event.event_type(), self.event_log.len());
    }
}

#[async_trait]
impl Runnable for Audit {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        self.add_event_to_log(event).await;
    }
}
