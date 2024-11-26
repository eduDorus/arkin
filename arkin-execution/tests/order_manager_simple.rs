// use std::sync::Arc;

// use test_log::test;

// use arkin_core::prelude::*;
// use arkin_execution::prelude::*;
// use arkin_portfolio::SingleStrategyPortfolioBuilder;

// #[test(tokio::test)]
// async fn test_place_execution_order() {
//     // Create mock Executor and Portfolio
//     // let pubsub = PubSub::new();
//     // let portfolio = SingleStrategyPortfolioBuilder::default().build().unwrap();

//     // Build the SingleExecutorOrderManager with mocks
//     // let order_manager = SimpleOrderManagerBuilder::default()
//     //     .pubsub(Arc::new(pubsub))
//     //     .portfolio(Arc::new(portfolio))
//     //     .build()
//     //     .unwrap();

//     // Create a test ExecutionOrder
//     // let instrument = test_inst_binance_btc_usdt_perp();
//     // let first_order = ExecutionOrderBuilder::default()
//     //     .instrument(instrument.clone())
//     //     .execution_type(ExecutionOrderStrategy::WideQuoting(
//     //         WideQuotingBuilder::default()
//     //             .spread_from_mid(dec!(0.025))
//     //             .requote_price_move_pct(dec!(0.005))
//     //             .build()
//     //             .unwrap(),
//     //     ))
//     //     .side(MarketSide::Buy)
//     //     .quantity(Quantity::from_f64(1.0).unwrap())
//     //     .build()
//     //     .unwrap();

//     // // Call place_order
//     // order_manager.place_order(first_order.clone()).await.unwrap();

//     // // Get the list of orders
//     // let new_orders = order_manager.list_new_orders().await;

//     // // Assert that the order is in the execution_orders map
//     // assert_eq!(new_orders.len(), 1);
//     // assert_eq!(new_orders[0].id, first_order.id);
// }
