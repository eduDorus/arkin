use std::sync::Arc;

use rust_decimal::prelude::*;
use test_log::test;

use arkin_core::prelude::*;
use arkin_execution::prelude::*;

#[test(tokio::test)]
async fn test_place_order() {
    // Create a mock OrderManager
    let mock_order_manager = MockOrderManager::new();

    // Build the SimulationExecutor with the mock OrderManager
    let executor = BacktestExecutorBuilder::default()
        .order_manager(Arc::new(mock_order_manager))
        .build()
        .unwrap();

    // // Create a sample VenueOrder
    let instrument = binance_btc_usdt_perp();
    let execution_order_id = ExecutionOrderId::new_v4();
    let order = VenueOrderBuilder::default()
        .execution_order_id(execution_order_id)
        .instrument(instrument)
        .order_type(VenueOrderType::Limit)
        .side(MarketSide::Buy)
        .quantity(Decimal::from_f64(0.1).unwrap())
        .price(Some(Decimal::from_f64(50000.).unwrap()))
        .build()
        .unwrap();

    // Call place_order
    executor.place_order(order.clone()).await.unwrap();
}
