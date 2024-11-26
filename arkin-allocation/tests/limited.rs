use std::sync::Arc;

use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use test_log::test;
use time::OffsetDateTime;

use arkin_allocation::prelude::*;
use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use arkin_portfolio::prelude::*;
use tracing::info;

#[test(tokio::test)]
async fn test_limited_allocation() {
    let mut persistence = MockPersistor::new();
    persistence.expect_read_latest_tick().returning(|_, _| {
        Ok(Some(
            TickBuilder::default()
                .event_time(OffsetDateTime::now_utc())
                .instrument(test_inst_binance_btc_usdt_perp())
                .tick_id(1234 as u64)
                .ask_price(dec!(51))
                .ask_quantity(dec!(1))
                .bid_price(dec!(49))
                .bid_quantity(dec!(1))
                .build()
                .expect("Failed to build Tick"),
        ))
    });

    let portfolio = MockPortfolio::new();
    // portfolio.expect_capital().returning(|| Decimal::from_f64(10000.0).unwrap());
    // portfolio.expect_positions().returning(|| HashMap::new());

    // Setup allocation
    let allocation_optim = LimitedAllocationOptimBuilder::default()
        .persistence(Arc::new(persistence))
        .portfolio(Arc::new(portfolio))
        .max_allocation(Decimal::from_f64(0.8).unwrap())
        .max_allocation_per_signal(Decimal::from_f64(0.1).unwrap())
        .build()
        .expect("Failed to build LimitedAllocationOptim");

    // Create signal
    let event_time = OffsetDateTime::now_utc();
    let signal = SignalBuilder::default()
        .event_time(event_time)
        .instrument(test_inst_binance_btc_usdt_perp())
        .strateg_id(Arc::new(String::from("CrossOver")))
        .weight(Decimal::from_f64(1.0).unwrap())
        .build()
        .expect("Failed to build Signal");

    // Add signal
    allocation_optim.new_signal(signal).await.unwrap();

    // Get signals
    let signals = allocation_optim.list_signals().await.unwrap();

    assert_eq!(signals.len(), 1);

    let orders = allocation_optim.optimize(event_time).await.unwrap();
    assert_eq!(orders.len(), 1);
    for order in orders {
        info!("{}", order);
    }
}
