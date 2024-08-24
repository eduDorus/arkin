use arkin_common::prelude::*;
use arkin_insights::prelude::*;
use arkin_persistance::prelude::*;
use arkin_strategies::prelude::*;
use ctor::ctor;
use rstest::*;
use time::OffsetDateTime;

#[ctor]
fn setup_env() {
    init_test_tracing();
    std::env::set_var("RUN_MODE", "test");
}

#[fixture]
pub fn instrument() -> Instrument {
    Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into())
}

#[fixture]
pub fn event_time() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

#[fixture]
pub fn database() -> DBManager {
    let config = load::<PersistanceConfig>();
    DBManager::from_config(&config.database)
}

#[fixture]
pub fn insights_manager() -> InsightsManager {
    let config = load::<InsightsConfig>();
    InsightsManager::from_config(&config.insights_manager)
}

#[fixture]
pub fn strategy_manager() -> StrategyManager {
    let config = load::<StrategyConfig>();
    StrategyManager::from_config(&config.strategy_manager)
}
