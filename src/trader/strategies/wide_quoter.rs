use std::sync::Arc;

use rust_decimal::Decimal;
use tracing::info;

use crate::{config::WideQuoterConfig, state::StateManager};

use super::Strategy;

#[derive(Clone)]
pub struct WideQuoter {
    state: Arc<StateManager>,
    spread_in_pct: Decimal,
}

impl WideQuoter {
    pub fn new(state: Arc<StateManager>, config: &WideQuoterConfig) -> Self {
        Self {
            state,
            spread_in_pct: config.spread_in_pct,
        }
    }
}

impl Strategy for WideQuoter {
    async fn start(&self) {
        info!("Starting wide quoter strategy...");
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            interval.tick().await;
            info!("Spreader takes snapshot");
        }
    }
}
