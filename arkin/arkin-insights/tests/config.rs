use arkin_common::prelude::*;
use arkin_insights::prelude::*;

#[test]
fn load_config() {
    init_test_tracing();
    std::env::set_var("RUN_MODE", "test");

    let config = load::<InsightsConfig>();
    assert_eq!(config.insights_manager.pipeline.frequency, 1);
}
