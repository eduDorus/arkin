use std::sync::Arc;

use arkin_portfolio::Portfolio;
use async_trait::async_trait;
use dashmap::DashMap;
use derive_builder::Builder;
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument, warn};

use arkin_core::prelude::*;

use crate::{OrderManager, OrderManagerError};

#[derive(Debug, Builder)]
pub struct SimpleOrderManager {
    pubsub: Arc<PubSub>,
    portfolio: Arc<dyn Portfolio>,
    #[builder(default = OrderQueue::default())]
    execution_orders: OrderQueue,
}

#[derive(Debug, Clone, Default)]
pub struct OrderQueue {
    orders: DashMap<ExecutionOrderId, ExecutionOrder>,
}

impl OrderQueue {
    pub fn get_order_by_id(&self, id: ExecutionOrderId) -> Option<ExecutionOrder> {
        self.orders.get(&id).map(|e| e.value().clone())
    }

    pub fn list_new_orders(&self) -> Vec<ExecutionOrder> {
        self.orders
            .iter()
            .filter(|e| e.value().is_new())
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn list_open_orders(&self) -> Vec<ExecutionOrder> {
        self.orders
            .iter()
            .filter(|e| !e.value().is_closed())
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn list_cancelling_orders(&self) -> Vec<ExecutionOrder> {
        self.orders
            .iter()
            .filter(|e| e.value().is_cancelling())
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn list_cancelled_orders(&self) -> Vec<ExecutionOrder> {
        self.orders
            .iter()
            .filter(|e| e.value().is_cancelled())
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn list_closed_orders(&self) -> Vec<ExecutionOrder> {
        self.orders
            .iter()
            .filter(|e| e.value().is_closed())
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn add_order(&self, order: ExecutionOrder) {
        self.orders.insert(order.id.clone(), order);
    }

    pub fn add_fill(&self, fill: Fill) {
        self.orders.alter(&fill.execution_order_id, |_, mut v| {
            if !v.is_closed() {
                v.add_fill(fill.clone());
            } else {
                warn!("Order {} is already closed but is getting a fill", fill.execution_order_id);
            }
            v
        });
    }

    pub fn update_order_status(&self, id: ExecutionOrderId, status: ExecutionOrderStatus) {
        if let Some(mut order) = self.orders.get_mut(&id) {
            if !order.is_closed() {
                order.update_status(status);
            }
        }
    }

    pub fn cancel_order_by_id(&self, id: ExecutionOrderId) {
        if let Some(mut entry) = self.orders.get_mut(&id) {
            let order = entry.value_mut();
            order.cancel();
        } else {
            warn!("No order found for id {}", id);
        }
    }

    pub fn cancel_orders_by_instrument(&self, instrument: &Arc<Instrument>) {
        let order_id = self
            .orders
            .iter()
            .find(|e| e.value().instrument == *instrument)
            .map(|e| e.key().clone());
        if let Some(id) = order_id {
            self.cancel_order_by_id(id);
        } else {
            warn!("No order found for instrument {}", instrument);
        }
    }

    pub fn cancel_all_orders(&self) {
        self.orders.alter_all(|_, mut v| {
            if !v.is_closed() {
                v.cancel();
            }
            v
        });
    }
}

#[async_trait]
impl OrderManager for SimpleOrderManager {
    #[instrument(skip_all)]
    async fn start(&self, shutdown: CancellationToken) -> Result<(), OrderManagerError> {
        info!("Starting order manager...");
        let mut execution_orders = self.pubsub.subscribe::<ExecutionOrder>();
        let mut fills = self.pubsub.subscribe::<Fill>();
        loop {
            tokio::select! {
                Ok(order) = execution_orders.recv() => {
                    match self.place_order(order).await {
                        Ok(_) => info!("Order processed"),
                        Err(e) => warn!("Failed to process order: {}", e),
                    }
                }
                Ok(fill) = fills.recv() => {
                    match self.order_update(fill).await {
                        Ok(_) => info!("Fill processed"),
                        Err(e) => warn!("Failed to process fill: {}", e),
                    }
                }
                _ = shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }

    #[instrument(skip_all)]
    async fn cleanup(&self) -> Result<(), OrderManagerError> {
        info!("Cleaning up order manager...");
        info!("Order manager cleaned up");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn order_by_id(&self, id: ExecutionOrderId) -> Option<ExecutionOrder> {
        self.execution_orders.get_order_by_id(id)
    }

    #[instrument(skip_all)]
    async fn list_new_orders(&self) -> Vec<ExecutionOrder> {
        self.execution_orders.list_new_orders()
    }

    #[instrument(skip_all)]
    async fn list_open_orders(&self) -> Vec<ExecutionOrder> {
        self.execution_orders.list_open_orders()
    }

    #[instrument(skip_all)]
    async fn list_cancelling_orders(&self) -> Vec<ExecutionOrder> {
        self.execution_orders.list_cancelling_orders()
    }

    #[instrument(skip_all)]
    async fn list_cancelled_orders(&self) -> Vec<ExecutionOrder> {
        self.execution_orders.list_cancelled_orders()
    }

    #[instrument(skip_all)]
    async fn list_closed_orders(&self) -> Vec<ExecutionOrder> {
        self.execution_orders.list_closed_orders()
    }

    #[instrument(skip_all)]
    async fn place_order(&self, order: ExecutionOrder) -> Result<(), OrderManagerError> {
        self.execution_orders.add_order(order.clone());
        Ok(())
    }

    #[instrument(skip_all)]
    async fn place_orders(&self, orders: Vec<ExecutionOrder>) -> Result<(), OrderManagerError> {
        for order in orders {
            self.place_order(order).await?;
        }
        Ok(())
    }

    #[instrument(skip_all)]
    async fn replace_order_by_id(&self, id: ExecutionOrderId, order: ExecutionOrder) -> Result<(), OrderManagerError> {
        self.execution_orders.cancel_order_by_id(id);
        self.execution_orders.add_order(order);
        Ok(())
    }

    #[instrument(skip_all)]
    async fn replace_orders_by_instrument(
        &self,
        instrument: &Arc<Instrument>,
        order: ExecutionOrder,
    ) -> Result<(), OrderManagerError> {
        self.execution_orders.cancel_orders_by_instrument(instrument);
        self.execution_orders.add_order(order);
        Ok(())
    }

    #[instrument(skip_all)]
    async fn cancel_order_by_id(&self, id: ExecutionOrderId) -> Result<(), OrderManagerError> {
        self.execution_orders.cancel_order_by_id(id);
        Ok(())
    }

    #[instrument(skip_all)]
    async fn cancel_orders_by_instrument(&self, instrument: &Arc<Instrument>) -> Result<(), OrderManagerError> {
        self.execution_orders.cancel_orders_by_instrument(instrument);
        Ok(())
    }

    #[instrument(skip_all)]
    async fn cancel_all_orders(&self) -> Result<(), OrderManagerError> {
        self.execution_orders.cancel_all_orders();
        Ok(())
    }

    #[instrument(skip_all)]
    async fn order_update(&self, fill: Fill) -> Result<(), OrderManagerError> {
        self.execution_orders.add_fill(fill);
        Ok(())
    }

    #[instrument(skip_all)]
    async fn order_status_update(
        &self,
        id: ExecutionOrderId,
        status: ExecutionOrderStatus,
    ) -> Result<(), OrderManagerError> {
        self.execution_orders.update_order_status(id, status);
        Ok(())
    }

    #[instrument(skip_all)]
    async fn position_update(&self, position: Position) -> Result<(), OrderManagerError> {
        self.portfolio.position_update(position).await?;
        Ok(())
    }

    #[instrument(skip_all)]
    async fn balance_update(&self, holding: Holding) -> Result<(), OrderManagerError> {
        self.portfolio.balance_update(holding).await?;
        Ok(())
    }
}
