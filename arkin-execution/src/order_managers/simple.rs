use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{OrderManager, OrderManagerError};

#[derive(Debug, TypedBuilder)]
pub struct SimpleOrderManager {
    pubsub: Arc<PubSub>,
    #[builder(default = OrderQueue::default())]
    execution_orders: OrderQueue,
}

#[derive(Debug, Clone, Default)]
pub struct OrderQueue {
    orders: DashMap<ExecutionOrderId, Arc<ExecutionOrder>>,
}

impl OrderQueue {
    pub fn get_order_by_id(&self, id: ExecutionOrderId) -> Option<Arc<ExecutionOrder>> {
        self.orders.get(&id).map(|e| e.value().clone())
    }

    pub fn list_new_orders(&self) -> Vec<Arc<ExecutionOrder>> {
        self.orders
            .iter()
            .filter(|e| e.value().is_new())
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn list_open_orders(&self) -> Vec<Arc<ExecutionOrder>> {
        self.orders
            .iter()
            .filter(|e| !e.value().is_closed())
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn list_cancelling_orders(&self) -> Vec<Arc<ExecutionOrder>> {
        self.orders
            .iter()
            .filter(|e| e.value().is_cancelling())
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn list_cancelled_orders(&self) -> Vec<Arc<ExecutionOrder>> {
        self.orders
            .iter()
            .filter(|e| e.value().is_cancelled())
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn list_closed_orders(&self) -> Vec<Arc<ExecutionOrder>> {
        self.orders
            .iter()
            .filter(|e| e.value().is_closed())
            .map(|e| e.value().clone())
            .collect()
    }

    pub fn add_order(&self, order: Arc<ExecutionOrder>) {
        self.orders.insert(order.id.clone(), order);
    }

    pub fn add_fill(&self, _fill: Arc<VenueOrderFill>) {
        // TODO
        // self.orders.alter(&fill.execution_order_id, |_, mut v| {
        //     if !v.is_closed() {
        //         v.add_fill(fill.clone());
        //     } else {
        //         warn!("Order {} is already closed but is getting a fill", fill.execution_order_id);
        //     }
        //     v
        // });
    }

    pub fn update_order_status(&self, _id: ExecutionOrderId, _status: ExecutionOrderStatus) {
        unimplemented!("Implement update_order_status");
        // if let Some(mut order) = self.orders.get_mut(&id) {
        //     if !order.is_closed() {
        //         order.update_status(status);
        //     }
        // }
    }

    pub fn cancel_order_by_id(&self, _id: ExecutionOrderId) {
        unimplemented!("Implement cancel_order_by_id");
        // if let Some(mut entry) = self.orders.get_mut(&id) {
        //     let order = entry.value_mut();
        //     order.cancel();
        // } else {
        //     warn!("No order found for id {}", id);
        // }
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
        unimplemented!("Implement cancel_all_orders");
        // self.orders.alter_all(|_, mut v| {
        //     if !v.is_closed() {
        //         v.cancel();
        //     }
        //     v
        // });
    }
}

#[async_trait]
impl OrderManager for SimpleOrderManager {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), OrderManagerError> {
        info!("Starting order manager...");
        let mut execution_orders = self.pubsub.subscribe::<ExecutionOrder>();
        let mut venue_order_updates = self.pubsub.subscribe::<VenueOrderUpdate>();
        loop {
            tokio::select! {
                Ok(order) = execution_orders.recv() => {
                    info!("SimpleOrderManager received order: {}", order.instrument);
                    if let Err(e) = self.place_order(order.clone()).await {
                        error!("Failed to process order: {}", e);
                    }
                    let venue_order = VenueOrder::builder()
                        .id(order.id)
                        .portfolio(test_portfolio())
                        .instrument(order.instrument.to_owned())
                        .side(order.side)
                        .order_type(order.order_type.into())
                        .price(None)
                        .quantity(order.quantity)
                        .build();

                    self.pubsub.publish::<VenueOrder>(venue_order.into());
                }
                Ok(order) = venue_order_updates.recv() => {
                    info!("SimpleOrderManager received order update: {}", order);
                    // if let Err(e) = self.order_update(fill.clone()).await {
                    //     error!("Failed to process fill: {}", e);
                    // }
                }
                _ = shutdown.cancelled() => {
                    break;
                }
            }
        }
        Ok(())
    }

    async fn order_by_id(&self, id: ExecutionOrderId) -> Option<Arc<ExecutionOrder>> {
        self.execution_orders.get_order_by_id(id)
    }

    async fn list_new_orders(&self) -> Vec<Arc<ExecutionOrder>> {
        self.execution_orders.list_new_orders()
    }

    async fn list_open_orders(&self) -> Vec<Arc<ExecutionOrder>> {
        self.execution_orders.list_open_orders()
    }

    async fn list_cancelling_orders(&self) -> Vec<Arc<ExecutionOrder>> {
        self.execution_orders.list_cancelling_orders()
    }

    async fn list_cancelled_orders(&self) -> Vec<Arc<ExecutionOrder>> {
        self.execution_orders.list_cancelled_orders()
    }

    async fn list_closed_orders(&self) -> Vec<Arc<ExecutionOrder>> {
        self.execution_orders.list_closed_orders()
    }

    async fn place_order(&self, order: Arc<ExecutionOrder>) -> Result<(), OrderManagerError> {
        self.execution_orders.add_order(order.clone());
        Ok(())
    }

    async fn place_orders(&self, orders: Vec<Arc<ExecutionOrder>>) -> Result<(), OrderManagerError> {
        for order in orders {
            self.place_order(order).await?;
        }
        Ok(())
    }

    async fn replace_order_by_id(
        &self,
        id: ExecutionOrderId,
        order: Arc<ExecutionOrder>,
    ) -> Result<(), OrderManagerError> {
        self.execution_orders.cancel_order_by_id(id);
        self.execution_orders.add_order(order);
        Ok(())
    }

    async fn replace_orders_by_instrument(
        &self,
        instrument: &Arc<Instrument>,
        order: Arc<ExecutionOrder>,
    ) -> Result<(), OrderManagerError> {
        self.execution_orders.cancel_orders_by_instrument(instrument);
        self.execution_orders.add_order(order);
        Ok(())
    }

    async fn cancel_order_by_id(&self, id: ExecutionOrderId) -> Result<(), OrderManagerError> {
        self.execution_orders.cancel_order_by_id(id);
        Ok(())
    }

    async fn cancel_orders_by_instrument(&self, instrument: &Arc<Instrument>) -> Result<(), OrderManagerError> {
        self.execution_orders.cancel_orders_by_instrument(instrument);
        Ok(())
    }

    async fn cancel_all_orders(&self) -> Result<(), OrderManagerError> {
        self.execution_orders.cancel_all_orders();
        Ok(())
    }

    async fn order_update(&self, fill: Arc<VenueOrderFill>) -> Result<(), OrderManagerError> {
        self.execution_orders.add_fill(fill);
        Ok(())
    }

    async fn order_status_update(
        &self,
        id: ExecutionOrderId,
        status: ExecutionOrderStatus,
    ) -> Result<(), OrderManagerError> {
        self.execution_orders.update_order_status(id, status);
        Ok(())
    }
}
