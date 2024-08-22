use rstest::*;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use std::sync::Arc;
use time::OffsetDateTime;

use crate::{
    config::{PortfolioManagerConfig, StateManagerConfig},
    models::{ExecutionOrder, Instrument, OrderSide, Position, PositionSide, StrategyId, Venue},
    portfolio::PortfolioManager,
    state::StateManager,
};

#[fixture]
pub fn instrument() -> Instrument {
    Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into())
}

#[fixture]
pub fn strategy() -> StrategyId {
    "test_strategy".into()
}

#[fixture]
pub fn state_manager() -> StateManager {
    StateManager::from_config(&StateManagerConfig::default())
}

#[fixture]
pub fn portfolio_manager(#[default(state_manager())] state: StateManager) -> PortfolioManager {
    let config = PortfolioManagerConfig {
        initial_capital: dec!(100000),
        leverage: dec!(1),
        initial_margin: dec!(0.2),
        maintenance_margin: dec!(0.15),
    };
    PortfolioManager::from_config(Arc::new(state), &config)
}

#[fixture]
pub fn market_execution_order(
    #[default(1)] id: u64,
    #[default(OffsetDateTime::now_utc())] event_time: OffsetDateTime,
    #[default(strategy())] strategy_id: StrategyId,
    #[default(instrument())] instrument: Instrument,
    #[default(OrderSide::Buy)] side: OrderSide,
    #[default(dec!(1))] quantity: Decimal,
) -> ExecutionOrder {
    ExecutionOrder::new_market_order(id, event_time, strategy_id, instrument, side, quantity.into())
}

#[fixture]
pub fn position_fixture(
    #[default(OffsetDateTime::now_utc())] event_time: OffsetDateTime,
    #[default(strategy())] strategy_id: StrategyId,
    #[default(instrument())] instrument: Instrument,
    #[default(PositionSide::Long)] side: PositionSide,
    #[default(dec!(0))] price: Decimal,
    #[default(dec!(0))] quantity: Decimal,
    #[default(dec!(0))] commission: Decimal,
) -> Position {
    Position::new(
        event_time,
        strategy_id,
        instrument,
        side,
        price.into(),
        quantity.into(),
        commission.into(),
    )
}
