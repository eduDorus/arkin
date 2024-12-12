// use std::sync::Arc;

// use rust_decimal::prelude::*;
// use test_log::test;

// use arkin_core::prelude::*;
// use arkin_execution::prelude::*;
// use uuid::Uuid;

// #[test(tokio::test)]
// async fn test_place_order() {
//     // Build the SimulationExecutor with the mock OrderManager
//     let pubsub = Arc::new(PubSub::new());
//     let executor = SimulationExecutor::builder().pubsub(pubsub).build();

//     // // Create a sample VenueOrder
//     let instrument = test_inst_binance_btc_usdt_perp();
//     let order = VenueOrder::builder()
//         .id(Uuid::new_v4())
//         .portfolio(test_portfolio())
//         .instrument(instrument)
//         .order_type(VenueOrderType::Limit)
//         .side(MarketSide::Buy)
//         .quantity(Decimal::from_f64(0.1).unwrap())
//         .price(Some(Decimal::from_f64(50000.).unwrap()))
//         .build();

//     // Call place_order
//     executor.place_order(order.into()).await.unwrap();
// }
