use rust_decimal::prelude::*;
use test_log::test;

use arkin_core::prelude::*;
use arkin_execution::prelude::*;

#[test(tokio::test)]
async fn test_place_order() {
    // Build the SimulationExecutor with the mock OrderManager
    let executor = SimulationExecutor::builder().build().unwrap();

    // // Create a sample VenueOrder
    let instrument = test_inst_binance_btc_usdt_perp();
    let order = VenueOrder::builder()
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
