use arkin_core::prelude::*;
use arkin_insights::prelude::*;

#[test]
fn load_config() {
    init_test_tracing();
    std::env::set_var("RUN_MODE", "test");

    let _ = load::<InsightsConfig>();
}
