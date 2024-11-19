use std::sync::Arc;

use rust_decimal::prelude::*;
use test_log::test;

use arkin_allocation::prelude::*;
use arkin_core::prelude::*;
use arkin_execution::prelude::*;
use arkin_portfolio::prelude::*;
use time::OffsetDateTime;

#[test(tokio::test)]
async fn test_limited_allocation() {
    let portfolio = Arc::new(MockPortfolio::new());
    let order_manager = Arc::new(MockOrderManager::new());

    // Setup allocation
    let allocation_optim = LimitedAllocationOptimBuilder::default()
        .portfolio(portfolio)
        .order_manager(order_manager)
        .max_allocation(Decimal::from_f64(0.8).unwrap())
        .max_allocation_per_signal(Decimal::from_f64(0.1).unwrap())
        .build()
        .expect("Failed to build LimitedAllocationOptim");

    // Create signal
    let signal = SignalBuilder::default()
        .event_time(OffsetDateTime::now_utc())
        .instrument(binance_btc_usdt_perp())
        .strateg_id(Arc::new(String::from("CrossOver")))
        .weight(Decimal::from_f64(1.0).unwrap())
        .build()
        .expect("Failed to build Signal");

    // Add signal
    allocation_optim.new_signal(signal).await.unwrap();

    // Get signals
    let signals = allocation_optim.list_signals().await.unwrap();

    assert_eq!(signals.len(), 1);
}
