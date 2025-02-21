use arkin_core::prelude::*;
use arkin_insights::prelude::*;

#[test]
fn load_config() {
    init_test_tracing();

    let _config = load::<InsightsConfig>();
}
