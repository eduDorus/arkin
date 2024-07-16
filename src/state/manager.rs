use time::Duration;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::{debug, error};

use crate::{
    config::StateConfig,
    features::FeatureEvent,
    models::{AccountEvent, MarketEvent},
};

use super::{
    market::StateData,
    order_manager::{OrderManagerType, SingleOrderManager},
    portfolio::{PortfolioType, SinglePortfolio},
};

#[allow(unused)]
pub struct StateManager {
    pub data: StateData,
    order_manager: OrderManagerType,
    portfolio: PortfolioType,
    market_sender: Sender<MarketEvent>,
    account_sender: Sender<AccountEvent>,
}

impl StateManager {
    pub fn new(_config: &StateConfig) -> Self {
        let market = StateData::default();
        let order_manager = OrderManagerType::SingleVenue(SingleOrderManager::new());
        let portfolio = PortfolioType::Single(SinglePortfolio::new());

        let (market_sender, _) = broadcast::channel(1024);
        let (account_sender, _) = broadcast::channel(1024);

        StateManager {
            data: market,
            order_manager,
            portfolio,
            market_sender,
            account_sender,
        }
    }

    pub async fn market_update(&self, event: &MarketEvent) {
        debug!("State received market event: {}", event);
        self.data.handle_market_event(event).await;
        if self.market_sender.receiver_count() > 0 {
            if let Err(e) = self.market_sender.send(event.to_owned()) {
                error!("Error sending market event: {}", e);
            }
        }
    }

    pub async fn account_update(&self, event: &AccountEvent) {
        debug!("State received account event: {}", event);
        self.data.handle_account_event(event).await;
    }

    pub async fn feature_update(&self, event: &FeatureEvent) {
        debug!("State received feature event: {}", event);
        self.data.handle_feature_event(event).await;
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
                tokio::time::sleep(std::time::Duration::from_secs(frequency.whole_seconds() as u64)).await;
                if let Err(e) = sender.send(()) {
                    error!("Error sending feature frequency event: {}", e);
                }
            }
        });

        receiver
    }
}
