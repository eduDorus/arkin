use crate::{config::BacktestExecutionConfig, models::AllocationEvent};
use rust_decimal::Decimal;

pub struct BacktestExecution {
    max_orders_per_minute: u64,
    max_order_size_notional: Decimal,
    min_order_size_notional: Decimal,
}

impl BacktestExecution {
    pub fn from_config(config: &BacktestExecutionConfig) -> Self {
        BacktestExecution {
            max_orders_per_minute: config.max_orders_per_minute,
            max_order_size_notional: config.max_order_size_notional,
            min_order_size_notional: config.min_order_size_notional,
        }
    }
}

impl BacktestExecution {
    fn update_allocation(&self, allocations: Vec<AllocationEvent>) {
        // Implementation goes here
    }
}
