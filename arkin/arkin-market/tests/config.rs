use arkin_common::prelude::*;
use arkin_market::prelude::*;

#[test]
fn load_config() {
    init_test_tracing();
    std::env::set_var("RUN_MODE", "test");

    let config = load::<MarketConfig>();
    assert_eq!(config.market_manager.lookback_min, 1440);
}
