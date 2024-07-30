use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;
use tracing::info;

use crate::{config::BinanceExecutionConfig, state::State};

use super::Execution;

#[derive(Clone)]
#[allow(unused)]
pub struct BinanceExecution {
    state: Arc<State>,
    max_orders_per_minute: u64,
    max_order_size_notional: Decimal,
    min_order_size_notional: Decimal,
}

impl BinanceExecution {
    pub fn new(state: Arc<State>, config: &BinanceExecutionConfig) -> Self {
        BinanceExecution {
            state,
            max_orders_per_minute: config.max_orders_per_minute,
            max_order_size_notional: config.max_order_size_notional,
            min_order_size_notional: config.min_order_size_notional,
        }
    }
}

#[async_trait]
impl Execution for BinanceExecution {
    async fn start(&self) {
        info!("Starting binance execution...");
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            interval.tick().await;
            info!("Executing binance orders...");
        }
    }
}
