use arkin_core::prelude::*;
use arkin_portfolio::prelude::*;
use rust_decimal::prelude::*;

#[test]
fn load_config() {
    init_test_tracing();
    std::env::set_var("RUN_MODE", "test");

    let config = load::<PortfolioConfig>();
    assert_eq!(config.portfolio_manager.initial_capital, Decimal::from_i64(100000).unwrap());
}
