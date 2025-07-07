use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use tracing::{error, info, instrument, warn};

use arkin_core::prelude::*;
use uuid::Uuid;

#[derive(Default)]
pub struct OrderBook {
    queue: DashMap<Uuid, VenueOrder>,
}

impl OrderBook {
    pub fn insert(&self, order: VenueOrder) {
        if !matches!(order.status, VenueOrderStatus::Inflight) {
            error!(target: "simulation-executor", "Adding order to order book that is not inflight");
        }
        self.queue.insert(order.id, order);
    }

    pub fn get(&self, id: Uuid) -> Option<VenueOrder> {
        self.queue.get(&id).map(|entry| entry.value().clone())
    }

    pub fn update(&self, order: VenueOrder) {
        if let Some(mut o) = self.queue.get_mut(&order.id) {
            *o = order;
        } else {
            error!(target: "simulation-executor", "Updating order that does not exist in the order book");
        }
    }

    pub fn remove(&self, id: Uuid) {
        self.queue.remove(&id);
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn list_orders(&self) -> Vec<VenueOrder> {
        let mut orders: Vec<VenueOrder> = self.queue.iter().map(|entry| entry.value().clone()).collect();
        orders.sort_by_key(|order| order.status);
        orders
    }

    pub fn list_orders_by_status(&self, status: VenueOrderStatus) -> Vec<VenueOrder> {
        self.queue
            .iter()
            .filter(|entry| entry.value().status == status)
            .map(|entry| entry.value().clone())
            .collect()
    }
}

pub struct SimulationExecution {
    identifier: String,
    _time: Arc<dyn SystemTime>,
    _publisher: Arc<dyn Publisher>,
    _orderbook: OrderBook,
}

impl SimulationExecution {
    pub fn new(identifier: &str, time: Arc<dyn SystemTime>, publisher: Arc<dyn Publisher>) -> Arc<Self> {
        Self {
            identifier: identifier.to_owned(),
            _time: time,
            _publisher: publisher,
            _orderbook: OrderBook::default(),
        }
        .into()
    }

    async fn place_order(&self, _order: &VenueOrder) {
        todo!()
        // tokio::time::sleep(Duration::from_millis(100)).await;
        // let time = self.time.now().await;
        // let mut order = order.clone();
        // if self.random.lock().await.random_bool(0.8) {
        //     info!(target: "executor", "order {} got placed by exchange", order.id());
        //     order.set_state(OrderState::Placed, time);
        //     let event = Event::OrderPlaced(order);
        //     self.publisher.publish(event).await;
        // } else {
        //     info!(target: "executor", "order {} got rejected by exchange", order.id());
        //     order.set_state(OrderState::Rejected, time);
        //     let event = Event::OrderRejected(order);
        //     self.publisher.publish(event).await;
        // }
    }

    async fn cancel_order(&self, _order: &VenueOrder) {
        todo!()
        // tokio::time::sleep(Duration::from_millis(100)).await;
        // info!(target: "executor", "order {} got cancelled by exchange", order.id());
        // let time = self.time.now().await;
        // let mut order = order.clone();
        // order.set_state(OrderState::Cancelled, time);
        // self.publisher.publish(Event::OrderCancelled(order)).await
    }

    async fn cancel_all(&self) {
        todo!()
    }
}

#[async_trait]
impl Runnable for SimulationExecution {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            Event::NewVenueOrder(o) => self.place_order(o).await,
            Event::CancelVenueOrder(o) => self.cancel_order(o).await,
            Event::CancelAllVenueOrders(_) => self.cancel_all().await,
            e => warn!(target: "executor", "received unused event {}", e),
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn start_tasks(self: Arc<Self>, _ctx: Arc<ServiceCtx>) {
        info!(target: "executor", "starting tasks");
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn stop_tasks(self: Arc<Self>, _ctx: Arc<ServiceCtx>) {
        info!(target: "executor", "stopping tasks");
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn teardown(&self, _ctx: Arc<ServiceCtx>) {
        info!(target: "executor", "teardown");
    }
}
