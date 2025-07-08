use std::sync::Arc;

use arkin_core::prelude::*;
use async_trait::async_trait;
use tracing::{info, instrument, warn};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct AllocationOptimizer {
    #[builder(default = String::from("allocation"))]
    identifier: String,
    _time: Arc<dyn SystemTime>,
    _publisher: Arc<dyn Publisher>,
    #[builder(default = Ledger::new())]
    _ledger: Arc<Ledger>,
}

impl AllocationOptimizer {
    async fn strategy_signal_update(&self, _signal: &Signal) {
        info!(target: "allocation", "received strategy signal");
    }
}

#[async_trait]
impl Runnable for AllocationOptimizer {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            Event::SignalUpdate(s) => self.strategy_signal_update(s).await,
            e => warn!(target: "allocation", "received unused event {}", e),
        }
    }
}
