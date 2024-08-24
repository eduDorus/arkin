use arkin_allocation::prelude::*;
use arkin_common::prelude::*;
use rust_decimal_macros::dec;

#[test]
fn load_config() {
    init_test_tracing();
    std::env::set_var("RUN_MODE", "test");

    let config = load::<AllocationConfig>();
    assert_eq!(config.allocation_manager.allocations.len(), 1);
    // Check if the config is a Crossover strategy
    let AllocationModuleConfig::Equal(c) = &config.allocation_manager.allocations[0];
    assert_eq!(c.max_allocation, dec!(0.9));
    assert_eq!(c.max_allocation_per_underlier, dec!(0.25));
}
