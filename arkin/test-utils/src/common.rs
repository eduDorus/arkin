use arkin_allocation::prelude::*;
use arkin_core::prelude::*;
use arkin_engine::{Engine, EngineBuilder};
use arkin_execution::prelude::*;
use arkin_insights::prelude::*;
use arkin_market::prelude::*;
use arkin_persistance::prelude::*;
use arkin_portfolio::prelude::*;
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
pub fn perpetual_btc() -> Instrument {
    Instrument::perpetual(Venue::Binance, "BTC".into(), "ETH".into())
}

#[fixture]
pub fn perpetual_eth() -> Instrument {
    Instrument::perpetual(Venue::Binance, "ETH".into(), "ETH".into())
}

#[fixture]
pub fn strategy_crossover() -> StrategyId {
    StrategyId::from("crossover")
}

#[fixture]
pub fn strategy_predator() -> StrategyId {
    StrategyId::from("predator")
}

#[fixture]
pub fn event_time() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

// #[fixture]
// fn base_fill(
//     #[default(event_time())] event_time: OffsetDateTime,
//     #[default(strategy_id())] strategy_id: StrategyId,
//     #[default(instrument())] instrument: Instrument,
//     #[default(1)] order_id: u64,
//     #[default(1)] venue_order_id: u64,
//     #[default(Side::Buy)] side: Side,
//     #[default(dec!(100.0))] price: Price,
//     #[default(dec!(1.0))] quantity: Quantity,
//     #[default(dec!(0.1))] commission: Notional,
// ) -> Fill {
//     Fill::new(
//         event_time,
//         strategy_id,
//         instrument,
//         order_id,
//         venue_order_id,
//         side,
//         price,
//         quantity,
//         commission,
//     )
// }

#[fixture]
pub fn database() -> DBManager {
    let config = load::<PersistanceConfig>();
    DBManager::from_config(&config.database)
}

#[fixture]
pub fn market_manager() -> MarketManager {
    let config = load::<MarketConfig>();
    MarketManager::from_config(&config.market_manager)
}

#[fixture]
pub fn portfolio_manager() -> PortfolioManager {
    let config = load::<PortfolioConfig>();
    PortfolioManager::from_config(&config.portfolio_manager)
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

#[fixture]
pub fn allocation_manager() -> AllocationManager {
    let config = load::<AllocationConfig>();
    AllocationManager::from_config(&config.allocation_manager)
}

#[fixture]
pub fn execution_manager() -> ExecutionManager {
    let config = load::<ExecutionConfig>();
    ExecutionManager::from_config(&config.execution_manager)
}

#[fixture]
pub fn engine(
    market_manager: MarketManager,
    portfolio_manager: PortfolioManager,
    insights_manager: InsightsManager,
    strategy_manager: StrategyManager,
    allocation_manager: AllocationManager,
    execution_manager: ExecutionManager,
) -> Engine {
    EngineBuilder::default()
        .market_manager(market_manager)
        .portfolio_manager(portfolio_manager)
        .insights_manager(insights_manager)
        .strategy_manager(strategy_manager)
        .allocation_manager(allocation_manager)
        .execution_manager(execution_manager)
        .build()
        .expect("Failed to build engine")
}
