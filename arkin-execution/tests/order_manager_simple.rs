use std::sync::Arc;

use rust_decimal::prelude::*;
use test_log::test;

use arkin_core::prelude::*;
use arkin_execution::prelude::*;
use arkin_portfolio::MockPortfolio;

#[test(tokio::test)]
async fn test_place_execution_order() {
    // Create mock Executor and Portfolio
    let mock_executor = MockExecutor::new();
    let mock_portfolio = MockPortfolio::new();

    // Build the SingleExecutorOrderManager with mocks
    let order_manager = SimpleOrderManagerBuilder::default()
        .executor(Arc::new(mock_executor))
        .portfolio(Arc::new(mock_portfolio))
        .build()
        .unwrap();

    // Create a test ExecutionOrder
    let instrument = binance_btc_usdt_perp();
    let execution_order = ExecutionOrderBuilder::default()
        .instrument(instrument.clone())
        .execution_type(ExecutionOrderStrategy::Market)
        .side(MarketSide::Buy)
        .quantity(Quantity::from_f64(1.0).unwrap())
        .build()
        .unwrap();

    // Call place_order
    order_manager.place_order(execution_order.clone()).await.unwrap();

    // Get the list of orders
    let orders = order_manager.list_new_orders().await;

    // Assert that the order is in the execution_orders map
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0], execution_order);

    // New execution order
    let execution_order = ExecutionOrderBuilder::default()
        .instrument(instrument.clone())
        .execution_type(ExecutionOrderStrategy::Market)
        .side(MarketSide::Sell)
        .quantity(Quantity::from_f64(1.0).unwrap())
        .build()
        .unwrap();

    // Call place_order
    order_manager
        .replace_orders_by_instrument(&instrument, execution_order.clone())
        .await
        .unwrap();

    // Get the list of orders
    let orders = order_manager.list_new_orders().await;

    // Assert that the order is in the execution_orders map
    assert_eq!(orders.len(), 1);
    assert_eq!(orders[0], execution_order);
}
