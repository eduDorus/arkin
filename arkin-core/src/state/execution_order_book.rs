use std::sync::Arc;

use dashmap::DashMap;
use time::UtcDateTime;
use tracing::{error, info};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{
    types::Commission, Event, ExecutionOrder, ExecutionOrderId, ExecutionOrderStatus, ExecutionStrategyType,
    Instrument, Price, Publisher, Quantity,
};

#[derive(TypedBuilder)]
pub struct ExecutionOrderBook {
    publisher: Arc<dyn Publisher>,
    queue: DashMap<ExecutionOrderId, ExecutionOrder>,
    #[builder(default = true)]
    autoclean: bool,
}

impl ExecutionOrderBook {
    pub fn new(publisher: Arc<dyn Publisher>, autoclean: bool) -> Arc<Self> {
        Self {
            publisher,
            queue: DashMap::new(),
            autoclean,
        }
        .into()
    }

    fn autoclean_order(&self, id: ExecutionOrderId) {
        if self.autoclean {
            let is_terminal = if let Some(order) = self.queue.get(&id) {
                order.is_terminal() // Evaluate and drop ref after this block.
            } else {
                false
            };
            if is_terminal {
                self.queue.remove(&id);
                info!(target: "exec_order_book", "auto cleanup removed finalized order {}", id);
            }
        }
    }

    pub async fn insert(&self, order: ExecutionOrder) {
        if !matches!(order.status, ExecutionOrderStatus::New) {
            error!(target: "exec_order_book", "Adding order to order book that is not new");
        }
        self.queue.insert(order.id, order.to_owned());

        info!(target: "exec_order_book", "inserted order {} in order book", order.id);
        self.publisher.publish(Event::ExecutionOrderBookNew(order.into())).await;
    }

    pub fn get(&self, id: Uuid) -> Option<ExecutionOrder> {
        self.queue.get(&id).map(|entry| entry.value().to_owned())
    }

    pub async fn place_order(&self, id: ExecutionOrderId, event_time: UtcDateTime) {
        if let Some(mut order) = self.queue.get_mut(&id) {
            order.place(event_time);
            info!(target: "exec_order_book", "placed order {} in exec order book", id);

            let update = order.clone();
            drop(order);
            self.publisher.publish(Event::ExecutionOrderBookUpdate(update.into())).await;
        } else {
            error!(target: "exec_order_book", "could not find order {} in exec order book", id);
        }
        self.autoclean_order(id);
    }

    pub async fn add_fill_to_order(
        &self,
        id: ExecutionOrderId,
        event_time: UtcDateTime,
        price: Price,
        quantity: Quantity,
        commission: Commission,
    ) {
        if let Some(mut order) = self.queue.get_mut(&id) {
            order.add_fill(event_time, price, quantity, commission);
            info!(target: "exec_order_book", "add fill to order {} from exec order book", id);

            let update = order.clone();
            drop(order);
            self.publisher.publish(Event::ExecutionOrderBookUpdate(update.into())).await;
        } else {
            error!(target: "exec_order_book", "could not find order {} in exec order book", id);
        }
        self.autoclean_order(id);
    }

    pub async fn cancel_order(&self, id: ExecutionOrderId, event_time: UtcDateTime) {
        if let Some(mut order) = self.queue.get_mut(&id) {
            order.cancel(event_time);
            info!(target: "exec_order_book", "cancelled order {} from exec order book", id);

            let update = order.clone();
            drop(order);
            self.publisher.publish(Event::ExecutionOrderBookUpdate(update.into())).await;
        } else {
            error!(target: "exec_order_book", "could not find order {} in exec order book", id);
        }
        self.autoclean_order(id);
    }

    pub async fn expire_order(&self, id: ExecutionOrderId, event_time: UtcDateTime) {
        if let Some(mut order) = self.queue.get_mut(&id) {
            order.expire(event_time);
            info!(target: "exec_order_book", "expired order {} from exec order book", id);

            let update = order.clone();
            drop(order);
            self.publisher.publish(Event::ExecutionOrderBookUpdate(update.into())).await;
        } else {
            error!(target: "exec_order_book", "could not find order {} in exec order book", id);
        }
        self.autoclean_order(id);
    }

    pub async fn reject_order(&self, id: ExecutionOrderId, event_time: UtcDateTime) {
        if let Some(mut order) = self.queue.get_mut(&id) {
            order.reject(event_time);

            info!(target: "exec_order_book", "rejected order {} from exec order book", id);
            let update = order.clone();
            drop(order);
            self.publisher.publish(Event::ExecutionOrderBookUpdate(update.into())).await;
        } else {
            error!(target: "exec_order_book", "could not find order {} in exec order book", id);
        }
        self.autoclean_order(id);
    }

    pub async fn finalize_terminate_order(&self, id: ExecutionOrderId, event_time: UtcDateTime) {
        info!(target: "exec_order_book", "check for terminating order {} from exec order book", id);
        if let Some(mut order) = self.queue.get_mut(&id) {
            let finalized = order.finalize_terminate(event_time);
            if finalized {
                info!(target: "exec_order_book", "finalized order {} in exec order book", id);

                let update = order.clone();
                drop(order);
                self.publisher.publish(Event::ExecutionOrderBookUpdate(update.into())).await;
            }
        } else {
            error!(target: "exec_order_book", "could not find order {} in exec order book", id);
        }
        self.autoclean_order(id);
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn list_ids(&self) -> Vec<ExecutionOrderId> {
        self.queue.iter().map(|entry| entry.key().to_owned()).collect()
    }

    pub fn list_ids_by_exec_strategy(&self, exec_type: ExecutionStrategyType) -> Vec<ExecutionOrderId> {
        self.queue
            .iter()
            .filter(|entry| entry.value().exec_strategy_type == exec_type)
            .map(|entry| entry.key().to_owned())
            .collect()
    }

    pub fn list_orders(&self) -> Vec<ExecutionOrder> {
        let mut orders: Vec<ExecutionOrder> = self.queue.iter().map(|entry| entry.value().to_owned()).collect();
        orders.sort_by_key(|order| order.status);
        orders
    }

    pub fn list_orders_by_exec_strategy(&self, exec_type: ExecutionStrategyType) -> Vec<ExecutionOrder> {
        self.queue
            .iter()
            .filter(|entry| entry.value().exec_strategy_type == exec_type)
            .map(|entry| entry.value().to_owned())
            .collect()
    }

    pub fn list_active_orders(&self) -> Vec<ExecutionOrder> {
        let mut orders: Vec<ExecutionOrder> = self
            .queue
            .iter()
            .filter(|entry| entry.value().is_active())
            .map(|entry| entry.value().to_owned())
            .collect();
        orders.sort_by_key(|order| order.status);
        orders
    }

    pub fn list_active_orders_by_instrument_and_strategy(
        &self,
        instrument: &Arc<Instrument>,
        exec_type: ExecutionStrategyType,
    ) -> Vec<ExecutionOrder> {
        self.queue
            .iter()
            .filter(|entry| {
                entry.value().is_active()
                    && entry.value().exec_strategy_type == exec_type
                    && &entry.value().instrument == instrument
            })
            .map(|entry| entry.value().to_owned())
            .collect()
    }

    pub fn list_orders_by_status(&self, status: ExecutionOrderStatus) -> Vec<ExecutionOrder> {
        self.queue
            .iter()
            .filter(|entry| entry.value().status == status)
            .map(|entry| entry.value().to_owned())
            .collect()
    }
}
