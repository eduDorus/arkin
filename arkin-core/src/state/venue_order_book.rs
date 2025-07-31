use std::sync::Arc;

use dashmap::DashMap;
use time::UtcDateTime;
use tracing::{error, info, warn};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{
    types::Commission, Event, ExecutionOrderId, Price, Publisher, Quantity, VenueOrder, VenueOrderId, VenueOrderStatus,
};

#[derive(TypedBuilder)]
pub struct VenueOrderBook {
    publisher: Arc<dyn Publisher>,
    queue: DashMap<VenueOrderId, VenueOrder>,
    #[builder(default = true)]
    autoclean: bool,
}

impl VenueOrderBook {
    pub fn new(publisher: Arc<dyn Publisher>, autoclean: bool) -> Arc<Self> {
        Self {
            publisher,
            queue: DashMap::new(),
            autoclean,
        }
        .into()
    }

    fn autoclean_order(&self, id: VenueOrderId) {
        if self.autoclean {
            let is_terminal = if let Some(order) = self.queue.get(&id) {
                order.is_terminal() // Evaluate and drop ref after this block.
            } else {
                false
            };
            if is_terminal {
                self.queue.remove(&id);
                info!(target: "venue_order_book", "auto cleanup removed finalized order {}", id);
            }
        }
    }

    pub async fn insert(&self, order: VenueOrder) {
        if !matches!(order.status, VenueOrderStatus::New) {
            error!(target: "venue_order_book", "Adding order to order book that is not new");
        }
        self.queue.insert(order.id, order.to_owned());

        info!(target: "venue_order_book", "inserted order {} in venue orderbook", order.id);
        self.publisher.publish(Event::VenueOrderBookNew(order.into())).await;
    }

    pub fn get(&self, id: Uuid) -> Option<VenueOrder> {
        self.queue.get(&id).map(|entry| entry.value().to_owned())
    }

    pub async fn set_inflight(&self, id: VenueOrderId, event_time: UtcDateTime) {
        if let Some(mut order) = self.queue.get_mut(&id) {
            order.set_inflight(event_time);
            info!(target: "venue_order_book", "inflight order {} in venue orderbook", id);

            let update = order.clone();
            drop(order);
            self.publisher.publish(Event::VenueOrderBookUpdate(update.into())).await;
        } else {
            error!(target: "venue_order_book", "could not find order {} in venue order book", id);
        }
        self.autoclean_order(id);
    }

    pub async fn place_order(&self, id: VenueOrderId, event_time: UtcDateTime) {
        if let Some(mut order) = self.queue.get_mut(&id) {
            order.place(event_time);
            info!(target: "venue_order_book", "placed order {} in venue order book", id);

            let update = order.clone();
            drop(order);
            self.publisher.publish(Event::VenueOrderBookUpdate(update.into())).await;
        } else {
            error!(target: "venue_order_book", "could not find order {} in venue order book", id);
        }
        self.autoclean_order(id);
    }

    pub async fn reject_order(&self, id: VenueOrderId, event_time: UtcDateTime) {
        if let Some(mut order) = self.queue.get_mut(&id) {
            order.reject(event_time);
            info!(target: "venue_order_book", "rejected order {} in venue order book", id);

            let update = order.clone();
            drop(order);
            self.publisher.publish(Event::VenueOrderBookUpdate(update.into())).await;
        } else {
            error!(target: "venue_order_book", "could not find order {} in venue order book", id);
        }
        self.autoclean_order(id);
    }

    pub async fn add_fill_to_order(
        &self,
        id: VenueOrderId,
        event_time: UtcDateTime,
        price: Price,
        quantity: Quantity,
        commission: Commission,
    ) {
        if let Some(mut order) = self.queue.get_mut(&id) {
            order.add_fill(event_time, price, quantity, commission);
            info!(target: "venue_order_book", "add fill to order {} in venue order book", id);

            let update = order.clone();
            drop(order);
            self.publisher.publish(Event::VenueOrderBookUpdate(update.into())).await;
        } else {
            error!(target: "venue_order_book", "could not find order {} in venue order book", id);
        }
        self.autoclean_order(id);
    }

    pub async fn cancel_order(&self, id: VenueOrderId, event_time: UtcDateTime) {
        if let Some(mut order) = self.queue.get_mut(&id) {
            order.cancel(event_time);
            info!(target: "venue_order_book", "cancelled order {} in venue order book", id);

            let update = order.clone();
            drop(order);
            self.publisher.publish(Event::VenueOrderBookUpdate(update.into())).await;
        } else {
            error!(target: "venue_order_book", "could not find order {} in venue order book", id);
        }
        self.autoclean_order(id);
    }

    pub async fn expire_order(&self, id: VenueOrderId, event_time: UtcDateTime) {
        if let Some(mut order) = self.queue.get_mut(&id) {
            order.expire(event_time);
            info!(target: "venue_order_book", "expired order {} in venue order book", id);

            let update = order.clone();
            drop(order);
            self.publisher.publish(Event::VenueOrderBookUpdate(update.into())).await;
        } else {
            error!(target: "venue_order_book", "could not find order {} in venue order book", id);
        }
        self.autoclean_order(id);
    }

    pub async fn check_finalize_order(&self, id: VenueOrderId, event_time: UtcDateTime) {
        if let Some(mut order) = self.queue.get_mut(&id) {
            let finalized = order.finalize_terminate(event_time);
            if finalized {
                info!(target: "venue_order_book", "finalized order {} in venue order book", id);

                let update = order.clone();
                drop(order);
                self.publisher.publish(Event::VenueOrderBookUpdate(update.into())).await;
            }
        } else {
            warn!(target: "venue_order_book", "could not find order {} in venue order book", id);
        }
        self.autoclean_order(id);
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
