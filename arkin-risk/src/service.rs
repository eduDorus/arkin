use std::sync::Arc;

use async_trait::async_trait;
use tracing::{instrument, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

#[derive(TypedBuilder)]
pub struct Risk {
    identifier: String,
    _time: Arc<dyn SystemTime>,
    _publisher: Arc<dyn Publisher>,
}

#[async_trait]
impl Runnable for Risk {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            // Event::InsightsTick(o) => self.place_order(o).await,
            e => warn!(target: "execution::binance", "received unused event {}", e),
        }
    }
}
