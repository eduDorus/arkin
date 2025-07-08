use std::sync::Arc;

use dashmap::DashMap;
use tracing::{error, warn};
use uuid::Uuid;

use crate::{VenueOrder, VenueOrderId, VenueOrderStatus};

#[derive(Default)]
pub struct VenueOrderBook {
    queue: DashMap<VenueOrderId, VenueOrder>,
}

impl VenueOrderBook {
    pub fn new() -> Arc<Self> {
        Self {
            queue: DashMap::new(),
        }
        .into()
    }

    pub fn insert(&self, order: VenueOrder) {
        if !matches!(order.status, VenueOrderStatus::New) {
            warn!(target: "order_manager", "Adding order to order book that is not new");
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
            error!(target: "order_manager", "Updating order that does not exist in the order book");
        }
    }

    pub fn remove(&self, id: Uuid) -> Option<(Uuid, VenueOrder)> {
        self.queue.remove(&id)
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
