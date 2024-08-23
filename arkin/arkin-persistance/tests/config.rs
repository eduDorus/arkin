use arkin_common::prelude::*;
use arkin_persistance::prelude::*;

#[test]
fn load_config() {
    init_test_tracing();
    // Set env var to test
    std::env::set_var("RUN_MODE", "test");

    let config = load::<PersistanceConfig>();
    assert_eq!(config.database.host, "127.0.0.1");
}
