use std::sync::Arc;

use arkin_core::prelude::*;
use async_trait::async_trait;
use tracing::{info, instrument, warn};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct Accounting {
    #[builder(default = String::from("accounting"))]
    identifier: String,
    _time: Arc<dyn SystemTime>,
    _publisher: Arc<dyn Publisher>,
    #[builder(default = Ledger::new())]
    _ledger: Arc<Ledger>,
}

impl Accounting {
    async fn venue_order_fill(&self, _order: &VenueOrder) {
        info!(target: "accounting", "received fill");
    }
}

#[async_trait]
impl Runnable for Accounting {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            Event::VenueOrderFill(vo) => self.venue_order_fill(vo).await,
            e => warn!(target: "accounting", "received unused event {}", e),
        }
    }
}
