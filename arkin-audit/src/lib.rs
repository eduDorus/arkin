use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use time::UtcDateTime;
use tracing::debug;

use arkin_core::prelude::*;

pub struct Audit {
    event_log: DashMap<UtcDateTime, Event>,
}

impl Default for Audit {
    fn default() -> Self {
        Self::new()
    }
}

impl Audit {
    pub fn new() -> Self {
        Self {
            event_log: DashMap::new(),
        }
    }

    pub fn event_log(&self) -> Vec<Event> {
        let mut events = self.event_log.iter().map(|e| e.value().clone()).collect::<Vec<_>>();
        events.sort_unstable();
        events
    }

    pub async fn add_event_to_log(&self, event: Event) {
        self.event_log.insert(event.timestamp(), event.to_owned());
        debug!(target: "audit", "add new event {} to log ({} logs)", event.event_type(), self.event_log.len());
    }
}

#[async_trait]
impl Runnable for Audit {
    async fn handle_event(&self, _core_ctx: Arc<CoreCtx>, event: Event) {
        self.add_event_to_log(event).await;
    }
}
