use std::sync::Arc;

use arkin_portfolio::Portfolio;
use async_trait::async_trait;
use dashmap::DashMap;
use derive_builder::Builder;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, instrument};

use arkin_core::prelude::*;

use crate::{Executor, OrderManager, OrderManagerError};

#[derive(Debug, Builder)]
pub struct SimpleOrderManager {
    executor: Arc<dyn Executor>,
    portfolio: Arc<dyn Portfolio>,
    #[builder(default = DashMap::new())]
    execution_orders: DashMap<Arc<Instrument>, ExecutionOrder>,
    #[builder(default = DashMap::new())]
    venue_orders: DashMap<VenueOrderId, VenueOrder>,
}

#[async_trait]
impl OrderManager for SimpleOrderManager {
    #[instrument(skip_all)]
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), OrderManagerError> {
        info!("Starting order manager...");
        self.executor.start(task_tracker.clone(), shutdown.clone()).await?;
        info!("Order manager started");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn cleanup(&self) -> Result<(), OrderManagerError> {
        info!("Cleaning up order manager...");
        self.executor.cleanup().await?;
        info!("Order manager cleaned up");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn list_orders(&self) -> Result<Vec<ExecutionOrder>, OrderManagerError> {
        Ok(self.execution_orders.iter().map(|x| x.value().clone()).collect())
    }

    #[instrument(skip_all)]
    async fn place_order(&self, order: ExecutionOrder) -> Result<(), OrderManagerError> {
        // Check if we have already an order for this instrument
        if self.execution_orders.contains_key(&order.instrument) {
            self.cancel_order(&order.instrument).await?;
        } else {
            self.execution_orders.insert(order.instrument.clone(), order);
        }
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
    async fn cancel_order(&self, instrument: &Arc<Instrument>) -> Result<(), OrderManagerError> {
        self.execution_orders.alter(instrument, |_, mut v| {
            v.cancel();
            v
        });
        Ok(())
    }

    #[instrument(skip_all)]
    async fn cancel_all_orders(&self) -> Result<(), OrderManagerError> {
        self.execution_orders.alter_all(|_, mut v| {
            v.cancel();
            v
        });
        Ok(())
    }

    #[instrument(skip_all)]
    async fn order_update(&self, fill: Fill) -> Result<(), OrderManagerError> {
        if let Some(mut order) = self.venue_orders.get_mut(&fill.venue_order_id) {
            order.add_fill(fill);
            Ok(())
        } else {
            Err(OrderManagerError::VenueOrderNotFound(fill.venue_order_id.to_string()))
        }
    }

    #[instrument(skip_all)]
    async fn order_status_update(&self, id: VenueOrderId, status: VenueOrderStatus) -> Result<(), OrderManagerError> {
        if let Some(mut order) = self.venue_orders.get_mut(&id) {
            order.update_status(status);
            Ok(())
        } else {
            Err(OrderManagerError::VenueOrderNotFound(id.to_string()))
        }
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
