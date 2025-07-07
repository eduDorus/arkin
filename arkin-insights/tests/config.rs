use arkin_core::prelude::*;
use arkin_insights::prelude::*;
use test_log::test;

#[test(test)]
fn load_config() {
    let _config = load::<InsightsConfig>();
}
