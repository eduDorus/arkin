use std::time::Duration;

use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, error};

use crate::{
    config::StateConfig,
    features::FeatureEvent,
    models::{AccountEvent, MarketEvent},
    state::{order_manager::OrderManager, portfolio::Portfolio},
};

use super::{
    market::MarketState,
    order_manager::{OrderManagerType, SingleOrderManager},
    portfolio::{PortfolioType, SinglePortfolio},
};

pub struct StateManager {
    market: MarketState,
    order_manager: OrderManagerType,
    portfolio: PortfolioType,
    market_sender: Sender<MarketEvent>,
    account_sender: Sender<AccountEvent>,
}

impl StateManager {
    pub fn new(_config: &StateConfig) -> Self {
        let market = MarketState::default();
        let order_manager = OrderManagerType::SingleVenue(SingleOrderManager::new());
        let portfolio = PortfolioType::Single(SinglePortfolio::new());

        let (market_sender, _) = broadcast::channel(1024);
        let (account_sender, _) = broadcast::channel(1024);

        StateManager {
            market,
            order_manager,
            portfolio,
            market_sender,
            account_sender,
        }
    }

    pub fn market_update(&self, event: &MarketEvent) {
        debug!("State received market event: {}", event);
        match event {
            MarketEvent::Tick(tick) => self.market.handle_tick_update(tick),
            MarketEvent::Trade(trade) => self.market.handle_trade_update(trade),
            MarketEvent::AggTrade(agg_trade) => self.market.handle_agg_trade_update(agg_trade),
            MarketEvent::BookUpdate(book_update) => self.market.handle_book_update(book_update),
        }
        if self.market_sender.receiver_count() > 0 {
            if let Err(e) = self.market_sender.send(event.to_owned()) {
                error!("Error sending market event: {}", e);
            }
        }
    }

    pub fn account_update(&self, event: &AccountEvent) {
        debug!("State received account event: {}", event);
        match event {
            AccountEvent::PositionUpdate(e) => self.portfolio.handle_position_update(e),
            AccountEvent::OrderUpdate(e) => self.order_manager.handle_order_update(e),
            AccountEvent::FillUpdate(e) => self.portfolio.handle_fill_update(e),
        }
    }

    pub fn feature_update(&self, event: &FeatureEvent) {
        debug!("State received feature event: {}", event);
        self.market.handle_feature_update(event);
    }

    pub fn listen_market_updates(&self) -> Receiver<MarketEvent> {
        self.market_sender.subscribe()
    }

    pub fn listen_account_updates(&self) -> Receiver<AccountEvent> {
        self.account_sender.subscribe()
    }

    pub fn listen_feature_frequency(&self, frequency: Duration) -> Receiver<()> {
        let (sender, receiver) = broadcast::channel(1);

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(frequency).await;
                if let Err(e) = sender.send(()) {
                    error!("Error sending feature frequency event: {}", e);
                }
            }
        });

        receiver
    }
}
