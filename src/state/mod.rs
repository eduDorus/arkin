use errors::StateError;
use market::MarketState;
use tracing::warn;
mod market;

use crate::{
    models::{AccountEvent, MarketEvent},
    order_manager::{OrderManager, OrderManagerType},
    portfolio::{Portfolio, PortfolioType},
};

pub mod errors;

pub struct State {
    market: MarketState,
    order_manager: OrderManagerType,
    portfolio: PortfolioType,
}

impl State {
    pub fn builder() -> StateBuilder {
        StateBuilder::default()
    }

    pub fn market_update(&mut self, event: &MarketEvent) {
        match event {
            MarketEvent::Tick(tick) => self.market.handle_tick_update(tick),
            MarketEvent::Trade(trade) => self.market.handle_trade_update(trade),
            MarketEvent::AggTrade(agg_trade) => self.market.handle_agg_trade_update(agg_trade),
        }
    }

    pub fn account_update(&mut self, event: &AccountEvent) {
        match event {
            AccountEvent::PositionUpdate(position) => self.portfolio.handle_position_update(position),
            AccountEvent::OrderUpdate(order) => self.order_manager.handle_order_update(order),
            _ => {
                warn!("Unhandled account event: {}", event)
            }
        }
    }
}

#[derive(Default)]
pub struct StateBuilder {
    order_manager: Option<OrderManagerType>,
    portfolio: Option<PortfolioType>,
}

impl StateBuilder {
    pub fn with_order_manager(mut self, order_manager: OrderManagerType) -> Self {
        self.order_manager = Some(order_manager);
        self
    }

    pub fn with_portfolio(mut self, portfolio: PortfolioType) -> Self {
        self.portfolio = Some(portfolio);
        self
    }

    pub fn build(self) -> Result<State, StateError> {
        Ok(State {
            market: MarketState::default(),
            order_manager: self
                .order_manager
                .ok_or(StateError::BuilderError("OrderManager not set".into()))?,
            portfolio: self.portfolio.ok_or(StateError::BuilderError("Portfolio not set".into()))?,
        })
    }
}
