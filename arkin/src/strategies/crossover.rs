use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;
use tracing::info;

use crate::{config::CrossoverConfig, state::State};

use super::Strategy;

#[derive(Clone)]
#[allow(unused)]
pub struct CrossoverStrategy {
    state: Arc<State>,
    fast: String,
    slow: String,
    min_spread: Decimal,
}

impl CrossoverStrategy {
    pub fn new(state: Arc<State>, config: &CrossoverConfig) -> Self {
        Self {
            state,
            fast: config.fast.to_owned(),
            slow: config.slow.to_owned(),
            min_spread: config.min_spread,
        }
    }
}

#[async_trait]
impl Strategy for CrossoverStrategy {
    async fn start(&self) {
        info!("Starting crossover strategy...");
        // let mut rx = self.state.subscribe_event(EventType::VWAP);

        // while let Ok(event) = rx.recv().await {
        //     info!("Wide quoter strategy received event: {}", event);
        //     match event {
        //         Event::VWAP(v) => {
        //             self.calculate_quote(v);
        //         }
        //         _ => {
        //             warn!("Wide quoter strategy received unused event: {}", event);
        //         }
        //     }
        // }
    }
}
