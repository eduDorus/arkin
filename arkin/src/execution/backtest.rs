use std::sync::Arc;

use async_trait::async_trait;
use rust_decimal::Decimal;
use tracing::info;

use crate::{config::BacktestExecutionConfig, state::State};

use super::Execution;

#[derive(Clone)]
#[allow(unused)]
pub struct BacktestExecution {
    state: Arc<State>,
    max_orders_per_minute: u64,
    max_order_size_notional: Decimal,
    min_order_size_notional: Decimal,
}

impl BacktestExecution {
    pub fn new(state: Arc<State>, config: &BacktestExecutionConfig) -> Self {
        BacktestExecution {
            state,
            max_orders_per_minute: config.max_orders_per_minute,
            max_order_size_notional: config.max_order_size_notional,
            min_order_size_notional: config.min_order_size_notional,
        }
    }
}

#[async_trait]
impl Execution for BacktestExecution {
    async fn start(&self) {
        info!("Starting backtest execution...");
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

        loop {
            interval.tick().await;
            info!("Execution tick...");
        }
    }
}
