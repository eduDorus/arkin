use arkin_common::prelude::*;
use arkin_portfolio::prelude::*;
use rust_decimal_macros::dec;

#[test]
fn load_config() {
    init_test_tracing();
    std::env::set_var("RUN_MODE", "test");

    let config = load::<PortfolioConfig>();
    assert_eq!(config.portfolio_manager.initial_capital, dec!(100000));
}
