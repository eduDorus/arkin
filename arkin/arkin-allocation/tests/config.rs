use arkin_allocation::prelude::*;
use arkin_common::prelude::*;
use rust_decimal_macros::dec;

#[test]
fn load_config() {
    init_test_tracing();
    std::env::set_var("RUN_MODE", "test");

    let config = load::<AllocationConfig>();
    // Check if the config is a Crossover strategy
    let AllocationModuleConfig::Simple(c) = &config.allocation_manager.module;
    assert_eq!(c.max_allocation, dec!(0.9));
    assert_eq!(c.max_allocation_per_underlier, dec!(0.25));
}
