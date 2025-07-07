use std::sync::Arc;

use async_trait::async_trait;
use tracing::{info, instrument};

use arkin_core::prelude::*;

pub struct Audit {
    identifier: String,
}

impl Audit {
    pub fn new(indentifier: &str) -> Arc<Self> {
        Arc::new(Self {
            identifier: indentifier.to_owned(),
        })
    }
}

#[async_trait]
impl Runnable for Audit {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Arc<Event>) {
        info!(target: "audit", "new event added to audit: {}", event);
    }
}
