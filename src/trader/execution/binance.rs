use std::sync::Arc;

use flume::Receiver;
use rust_decimal::Decimal;
use tracing::info;

use crate::{config::BinanceExecutionConfig, state::State};

use super::{Execution, ExecutionEvent};

#[derive(Clone)]
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

impl Execution for BinanceExecution {
    async fn start(&self, receiver: Receiver<ExecutionEvent>) {
        info!("Starting binance execution");
        while let Ok(event) = receiver.recv_async().await {
            info!("Binance execution received event: {}", event);
        }
    }
}
