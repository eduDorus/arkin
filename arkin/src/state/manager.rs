use std::{sync::Arc, time::Duration};

use time::OffsetDateTime;
use tokio::sync::broadcast::{self, Receiver, Sender};

use crate::{
    config::StateConfig,
    models::{Event, EventType},
};

use super::{
    order_manager::{OrderManagerType, SingleOrderManager},
    portfolio::{PortfolioType, SinglePortfolio},
    store::DataStore,
    time_component::TimeComponent,
};

#[allow(unused)]
pub struct StateManager {
    pub data: DataStore,
    order_manager: OrderManagerType,
    portfolio: PortfolioType,
    event_update: Sender<Event>,
    time_component: Arc<TimeComponent>,
}

impl StateManager {
    pub fn new(config: &StateConfig) -> Self {
        let data = DataStore::default();
        let order_manager = OrderManagerType::SingleVenue(SingleOrderManager::new());
        let portfolio = PortfolioType::Single(SinglePortfolio::new());
        let time_component = Arc::new(TimeComponent::new(&config.time_component));

        let time_component_ref = time_component.clone();
        tokio::spawn(async move {
            time_component_ref.start().await;
        });
        let (event_update, _) = broadcast::channel(1024);

        StateManager {
            data,
            order_manager,
            portfolio,
            event_update,
            time_component,
        }
    }

    pub async fn event_update(&self, event: Event) {
        self.data.add_event(event).await;
    }

    pub fn subscribe_frequency(&self, frequency: Duration) -> Receiver<OffsetDateTime> {
        self.time_component.subscribe(frequency)
    }

    pub fn subscribe_event(&self, event_id: EventType) -> Receiver<Event> {
        self.data.subscribe(event_id)
    }
}
