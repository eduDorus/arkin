use std::sync::Arc;

use rust_decimal::Decimal;
use tracing::info;

use crate::{config::SpreaderConfig, state::StateManager};

use super::Strategy;

#[derive(Clone)]
pub struct Spreader {
    state: Arc<StateManager>,
    spread_in_pct: Decimal,
}

impl Spreader {
    pub fn new(state: Arc<StateManager>, config: &SpreaderConfig) -> Self {
        Self {
            state,
            spread_in_pct: config.min_spread,
        }
    }
}

impl Strategy for Spreader {
    async fn start(&self) {
        info!("Starting spreader strategy...");
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            interval.tick().await;
            info!("Spreader takes snapshot");
        }
    }
}
