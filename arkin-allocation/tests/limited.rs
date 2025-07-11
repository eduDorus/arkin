// #[test(tokio::test)]
// async fn test_limited_allocation() {
//     let mut persistence = MockPersistor::new();
//     persistence.tick_store.get_last_tick().returning(|_, _| {
//         Ok(Some(
//             Tick::builder()
//                 .event_time(UtcDateTime::now())
//                 .instrument(test_inst_binance_btc_usdt_perp())
//                 .tick_id(1234 as u64)
//                 .ask_price(dec!(51))
//                 .ask_quantity(dec!(1))
//                 .bid_price(dec!(49))
//                 .bid_quantity(dec!(1))
//                 .build()
//                 .expect("Failed to build Tick"),
//         ))
//     });

//     let portfolio = MockAccounting::new();
//     // portfolio.expect_capital().returning(|| Decimal::from_f64(10000.0).unwrap());
//     // portfolio.expect_positions().returning(|| HashMap::new());

//     // Setup allocation
//     let allocation_optim = LimitedAllocationOptim::builder()
//         .persistence(Arc::new(persistence))
//         .portfolio(Arc::new(portfolio))
//         .max_allocation(Decimal::from_f64(0.8).unwrap())
//         .max_allocation_per_signal(Decimal::from_f64(0.1).unwrap())
//         .build()
//         .expect("Failed to build LimitedAllocationOptim");

//     // Create signal
//     let event_time = UtcDateTime::now();
//     let signal = Signal::builder()
//         .event_time(event_time)
//         .instrument(test_inst_binance_btc_usdt_perp())
//         .strategy(test_strategy())
//         .weight(Decimal::from_f64(1.0).unwrap())
//         .build()
//         .expect("Failed to build Signal");

//     // Add signal
//     allocation_optim.new_signal(signal).await.unwrap();

//     // Get signals
//     let signals = allocation_optim.list_signals().await.unwrap();

//     assert_eq!(signals.len(), 1);

//     let orders = allocation_optim.optimize(event_time).await.unwrap();
//     assert_eq!(orders.len(), 1);
//     for order in orders {
//         info!("{}", order);
//     }
// }
