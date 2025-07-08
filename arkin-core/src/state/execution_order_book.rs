use std::sync::Arc;

use dashmap::DashMap;
use tracing::{error, info, warn};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{ExecutionOrder, ExecutionOrderId, ExecutionOrderStatus, ExecutionStrategyType};

#[derive(Default, TypedBuilder)]
pub struct ExecutionOrderBook {
    queue: DashMap<ExecutionOrderId, ExecutionOrder>,
    #[builder(default = true)]
    autoclean: bool,
}

impl ExecutionOrderBook {
    pub fn new(autoclean: bool) -> Arc<Self> {
        Self {
            queue: DashMap::new(),
            autoclean,
        }
        .into()
    }

    pub fn insert(&self, order: ExecutionOrder) {
        if !matches!(order.status, ExecutionOrderStatus::New) {
            warn!(target: "exec_order_book", "Adding order to order book that is not new");
        }
        self.queue.insert(order.id, order.to_owned());
        info!(target: "exec_order_book", "inserted order {} in order book", order.id);
    }

    pub fn get(&self, id: Uuid) -> Option<ExecutionOrder> {
        self.queue.get(&id).map(|entry| entry.value().to_owned())
    }

    pub fn update(&self, order: ExecutionOrder) {
        let remove = if let Some(mut o) = self.queue.get_mut(&order.id) {
            *o = order.to_owned();
            info!(target: "exec_order_book", "updated order {} in order book to {}", order.id, order.status);

            // autoclean
            if self.autoclean && o.is_finalized() {
                true
            } else {
                false
            }
        } else {
            error!(target: "exec_order_book", "Updating order that does not exist in the order book");
            false
        };

        if remove {
            self.queue.remove(&order.id);
            info!(target: "exec_order_book", "auto cleanup removed finalized order {} with state {}", order.id, order.status);
        }
    }

    pub fn remove(&self, id: Uuid) -> Option<(Uuid, ExecutionOrder)> {
        let entry = self.queue.remove(&id);
        info!(target: "exec_order_book", "removed order {} from order book", id);
        entry
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

    pub fn list_orders_by_status(&self, status: ExecutionOrderStatus) -> Vec<ExecutionOrder> {
        self.queue
            .iter()
            .filter(|entry| entry.value().status == status)
            .map(|entry| entry.value().to_owned())
            .collect()
    }
}
