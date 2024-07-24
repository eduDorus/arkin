use std::sync::Arc;

use rust_decimal::Decimal;
use tracing::info;

use crate::{config::WideQuoterConfig, models::EventID, state::StateManager};

use super::Strategy;

#[derive(Clone)]
#[allow(unused)]
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
        let mut rx = self.state.subscribe_event(EventID::VWAP);

        while let Ok(event) = rx.recv().await {
            info!("Wide quoter strategy received event: {}", event);
        }
    }
}
