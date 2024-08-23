use arkin_common::prelude::*;
use arkin_persistance::prelude::*;
use rstest::*;
use time::macros::datetime;

#[fixture]
pub fn database() -> DBManager {
    init_test_tracing();
    // Set env var to test
    std::env::set_var("RUN_MODE", "test");

    let config = load::<PersistanceConfig>();
    DBManager::from_config(&config.database)
}

#[rstest]
#[tokio::test]
async fn test_read_ticks(database: DBManager) {
    let from = datetime!(2024-07-01 00:00).assume_utc();
    let till = datetime!(2024-07-01 00:10).assume_utc();
    let ticks = database.read_ticks(&from, &till).await;
    assert_eq!(ticks.len(), 109151);
}

#[rstest]
#[tokio::test]
async fn test_read_trades(database: DBManager) {
    let from = datetime!(2024-07-01 00:00).assume_utc();
    let till = datetime!(2024-07-01 00:10).assume_utc();
    let ticks = database.read_trades(&from, &till).await;
    assert_eq!(ticks.len(), 8706);
}
