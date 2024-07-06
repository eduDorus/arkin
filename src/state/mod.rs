pub mod errors;
mod market;
mod order_manager;
mod portfolio;

use market::MarketState;
use order_manager::{OrderManager, OrderManagerType};
use portfolio::{Portfolio, PortfolioType};
use tracing::warn;

use crate::{
    config::StateConfig,
    models::{AccountEvent, MarketEvent},
};

pub struct State {
    market: MarketState,
    order_manager: OrderManagerType,
    portfolio: PortfolioType,
}

impl State {
    pub fn new(_config: &StateConfig) -> Self {
        let market = MarketState::default();
        let order_manager = OrderManagerType::SingleVenue(order_manager::SingleOrderManager::new());
        let portfolio = PortfolioType::Single(portfolio::SinglePortfolio::new());

        State {
            market,
            order_manager,
            portfolio,
        }
    }

    pub fn market_update(&self, event: &MarketEvent) {
        match event {
            MarketEvent::Tick(tick) => self.market.handle_tick_update(tick),
            MarketEvent::Trade(trade) => self.market.handle_trade_update(trade),
            MarketEvent::AggTrade(agg_trade) => self.market.handle_agg_trade_update(agg_trade),
            MarketEvent::BookUpdate(book_update) => self.market.handle_book_update(book_update),
        }
    }

    pub fn account_update(&self, event: &AccountEvent) {
        match event {
            AccountEvent::PositionUpdate(position) => self.portfolio.handle_position_update(position),
            AccountEvent::OrderUpdate(order) => self.order_manager.handle_order_update(order),
            _ => {
                warn!("Unhandled account event: {}", event)
            }
        }
    }
}
