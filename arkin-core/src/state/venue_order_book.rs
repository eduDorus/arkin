use std::sync::Arc;

use dashmap::DashMap;
use tracing::{error, info, warn};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{ExecutionOrderId, VenueOrder, VenueOrderId, VenueOrderStatus};

#[derive(Default, TypedBuilder)]
pub struct VenueOrderBook {
    queue: DashMap<VenueOrderId, VenueOrder>,
    #[builder(default = true)]
    autoclean: bool,
}

impl VenueOrderBook {
    pub fn new(autoclean: bool) -> Arc<Self> {
        Self {
            queue: DashMap::new(),
            autoclean,
        }
        .into()
    }

    pub fn insert(&self, order: VenueOrder) {
        if !matches!(order.status, VenueOrderStatus::New) {
            warn!(target: "venue_order_book", "Adding order to order book that is not new");
        }
        self.queue.insert(order.id, order.to_owned());
        info!(target: "venue_order_book", "inserted order {} in venue orderbook", order.id);
    }

    pub fn get(&self, id: Uuid) -> Option<VenueOrder> {
        self.queue.get(&id).map(|entry| entry.value().to_owned())
    }

    pub fn update(&self, order: VenueOrder) {
        let remove = if let Some(mut o) = self.queue.get_mut(&order.id) {
            *o = order.to_owned();
            info!(target: "venue_order_book", "updated order {} in venue orderbook to {}", order.id, order.status);

            // autoclean
            if self.autoclean && o.is_finalized() {
                true
            } else {
                false
            }
        } else {
            error!(target: "venue_order_book", "Updating order that does not exist in the order book");
            false
        };

        if remove {
            self.queue.remove(&order.id);
            info!(target: "venue_order_book", "auto cleanup removed finalized order {} with state {}", order.id, order.status);
        }
    }

    pub fn remove(&self, id: Uuid) -> Option<(Uuid, VenueOrder)> {
        let entry = self.queue.remove(&id);
        info!(target: "venue_order_book", "removed order {} in venue orderbook", id);
        entry
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn list_orders(&self) -> Vec<VenueOrder> {
        let mut orders: Vec<VenueOrder> = self.queue.iter().map(|entry| entry.value().to_owned()).collect();
        orders.sort_by_key(|order| order.status);
        orders
    }

    pub fn list_orders_by_exec_id(&self, id: ExecutionOrderId) -> Vec<VenueOrder> {
        self.queue
            .iter()
            .filter(|entry| entry.value().execution_order_id == Some(id))
            .map(|entry| entry.value().to_owned())
            .collect()
    }

    pub fn list_orders_by_status(&self, status: VenueOrderStatus) -> Vec<VenueOrder> {
        self.queue
            .iter()
            .filter(|entry| entry.value().status == status)
            .map(|entry| entry.value().to_owned())
            .collect()
    }
}
