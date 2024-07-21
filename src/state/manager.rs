use time::Duration;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tracing::error;

use crate::{config::StateConfig, models::Event};

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
    event_update: Sender<Event>,
}

impl StateManager {
    pub fn new(_config: &StateConfig) -> Self {
        let data = StateData::default();
        let order_manager = OrderManagerType::SingleVenue(SingleOrderManager::new());
        let portfolio = PortfolioType::Single(SinglePortfolio::new());

        let (event_update, _) = broadcast::channel(1024);

        StateManager {
            data,
            order_manager,
            portfolio,
            event_update,
        }
    }

    pub async fn market_update(&self, event: Event) {
        self.data.add_event(event).await;
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
