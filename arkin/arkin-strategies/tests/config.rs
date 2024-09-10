use arkin_core::prelude::*;
use arkin_strategies::prelude::*;

#[test]
fn load_config() {
    init_test_tracing();
    std::env::set_var("RUN_MODE", "test");

    let config = load::<StrategyConfig>();
    assert_eq!(config.strategy_manager.strategies.len(), 1);
    // Check if the config is a Crossover strategy
    let StrategyModuleConfig::Crossover(crossover) = &config.strategy_manager.strategies[0];
    assert_eq!(crossover.id, StrategyId::from("crossover"));
    assert_eq!(crossover.price_spread_id, FeatureId::from("spread_sma_vwap"));
    assert_eq!(crossover.volume_spread_id, FeatureId::from("spread_sma_volume"));
}
