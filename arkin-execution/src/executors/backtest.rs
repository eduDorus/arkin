use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use dashmap::DashMap;
use derive_builder::Builder;
use rust_decimal::prelude::*;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{info, instrument};

use arkin_core::prelude::*;

use crate::{Executor, ExecutorError, OrderManager};

#[derive(Debug, Builder)]
pub struct BacktestExecutor {
    order_manager: Arc<dyn OrderManager>,
    #[builder(default)]
    orders: DashMap<VenueOrderId, VenueOrder>,
}

#[async_trait]
impl Executor for BacktestExecutor {
    #[instrument(skip_all)]
    async fn start(&self, tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), ExecutorError> {
        info!("Starting simulation executor...");
        let order_manager = self.order_manager.clone();
        let orders = self.orders.clone();
        tracker.spawn(async move {
            loop {
                if shutdown.is_cancelled() {
                    info!("Stopping SimExecutor...");
                    break;
                }
                // Generate random number between 0-3s
                let delay = rand::random::<u64>() % 4;
                tokio::time::sleep(Duration::from_secs(delay)).await;
                // info!("Executing orders");

                // Check if any orders are ready to be executed
                for mut entry in orders.iter_mut() {
                    let order = entry.value_mut();
                    match order.status {
                        VenueOrderStatus::New => {
                            order.status = VenueOrderStatus::Placed;
                            // let _ = order_manager.order_status_update(order.id, VenueOrderStatus::Placed).await;
                        }
                        VenueOrderStatus::Placed => {
                            let fill = FillBuilder::default()
                                .venue_order_id(order.id)
                                .price(order.price.unwrap_or(Decimal::ZERO))
                                .quantity(order.quantity)
                                .build()
                                .unwrap();
                            let _ = order_manager.order_update(fill).await;
                            order.status = VenueOrderStatus::Filled;
                        }
                        _ => {}
                    }
                }

                // Remove filled orders
                orders.retain(|_, order| {
                    order.status != VenueOrderStatus::Filled
                        || order.status != VenueOrderStatus::Cancelled
                        || order.status != VenueOrderStatus::Rejected
                });
            }
        });
        info!("Simulation executor started");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn cleanup(&self) -> Result<(), ExecutorError> {
        info!("Cleaning up simulation executor...");
        info!("Simulation executor cleaned up");
        Ok(())
    }

    #[instrument(skip_all)]
    async fn place_order(&self, order: VenueOrder) -> Result<(), ExecutorError> {
        info!("Placing order: {:?}", order);
        self.orders.insert(order.id, order);
        Ok(())
    }

    #[instrument(skip_all)]
    async fn place_orders(&self, orders: Vec<VenueOrder>) -> Result<(), ExecutorError> {
        info!("Placing orders: {:?}", orders);
        for order in orders {
            self.orders.insert(order.id, order);
        }
        Ok(())
    }

    #[instrument(skip_all)]
    async fn modify_order(&self, _order: VenueOrder) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::modify_order")
    }

    #[instrument(skip_all)]
    async fn modify_orders(&self, _orders: Vec<VenueOrder>) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::modify_orders")
    }

    #[instrument(skip_all)]
    async fn cancel_order(&self, id: VenueOrderId) -> Result<(), ExecutorError> {
        info!("Cancelling order: {:?}", id);
        if let Some(mut order) = self.orders.get_mut(&id) {
            order.cancel();
            Ok(())
        } else {
            return Err(ExecutorError::InvalidOrder(id.to_string()));
        }
    }

    #[instrument(skip_all)]
    async fn cancel_orders(&self, ids: Vec<VenueOrderId>) -> Result<(), ExecutorError> {
        info!("Cancelling orders: {:?}", ids);
        for id in ids {
            if let Some(mut order) = self.orders.get_mut(&id) {
                order.cancel();
            } else {
                return Err(ExecutorError::InvalidOrder(id.to_string()));
            }
        }
        Ok(())
    }

    #[instrument(skip_all)]
    async fn cancel_all_orders(&self) -> Result<(), ExecutorError> {
        info!("Cancelling all orders");
        for mut order in self.orders.iter_mut() {
            order.cancel();
        }
        Ok(())
    }
}
