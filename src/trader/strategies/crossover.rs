use std::sync::Arc;

use rust_decimal::Decimal;
use tracing::{info, warn};

use crate::{
    config::CrossoverConfig,
    features::VWAP,
    models::{Event, EventID},
    state::StateManager,
};

use super::Strategy;

#[derive(Clone)]
#[allow(unused)]
pub struct CrossoverStrategy {
    state: Arc<StateManager>,
    fast: String,
    slow: String,
    min_spread: Decimal,
}

impl CrossoverStrategy {
    pub fn new(state: Arc<StateManager>, config: &CrossoverConfig) -> Self {
        Self {
            state,
            fast: config.fast.to_owned(),
            slow: config.slow.to_owned(),
            min_spread: config.min_spread,
        }
    }

    fn calculate_quote(&self, _vwap: VWAP) {}
}

impl Strategy for CrossoverStrategy {
    async fn start(&self) {
        info!("Starting wide quoter strategy...");
        let mut rx = self.state.subscribe_event(EventID::VWAP);

        while let Ok(event) = rx.recv().await {
            info!("Wide quoter strategy received event: {}", event);
            match event {
                Event::VWAP(v) => {
                    self.calculate_quote(v);
                }
                _ => {
                    warn!("Wide quoter strategy received unused event: {}", event);
                }
            }
        }
    }
}
