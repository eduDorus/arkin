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
pub struct SimulationExecutor {
    order_manager: Arc<dyn OrderManager>,
    #[builder(default)]
    orders: DashMap<VenueOrderId, VenueOrder>,
}

#[async_trait]
impl Executor for SimulationExecutor {
    #[instrument(skip(self))]
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
                            let _ = order_manager.order_status_update(order.id, VenueOrderStatus::Placed).await;
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

    #[instrument(skip(self))]
    async fn cleanup(&self) -> Result<(), ExecutorError> {
        info!("Cleaning up simulation executor...");
        info!("Simulation executor cleaned up");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn place_order(&self, order: VenueOrder) -> Result<(), ExecutorError> {
        info!("Placing order: {:?}", order);
        self.orders.insert(order.id, order);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn place_orders(&self, orders: Vec<VenueOrder>) -> Result<(), ExecutorError> {
        info!("Placing orders: {:?}", orders);
        for order in orders {
            self.orders.insert(order.id, order);
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn modify_order(&self, _order: VenueOrder) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::modify_order")
    }

    #[instrument(skip(self))]
    async fn modify_orders(&self, _orders: Vec<VenueOrder>) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::modify_orders")
    }

    #[instrument(skip(self))]
    async fn cancel_order(&self, id: VenueOrderId) -> Result<(), ExecutorError> {
        info!("Cancelling order: {:?}", id);
        if let Some(mut order) = self.orders.get_mut(&id) {
            order.cancel();
            Ok(())
        } else {
            return Err(ExecutorError::InvalidOrder(id.to_string()));
        }
    }

    #[instrument(skip(self))]
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

    #[instrument(skip(self))]
    async fn cancel_all_orders(&self) -> Result<(), ExecutorError> {
        info!("Cancelling all orders");
        for mut order in self.orders.iter_mut() {
            order.cancel();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockOrderManager;
    use arkin_core::test_utils::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_place_order() {
        // Create a mock OrderManager
        let mock_order_manager = MockOrderManager::new();

        // Build the SimulationExecutor with the mock OrderManager
        let executor = SimulationExecutorBuilder::default()
            .order_manager(Arc::new(mock_order_manager))
            .build()
            .unwrap();

        info!("Executor: {:?}", executor);

        // // Create a sample VenueOrder
        let instrument = binance_btc_usdt_perp();
        let execution_order_id = ExecutionOrderId::new_v4();
        let order = VenueOrderBuilder::default()
            .execution_order_id(execution_order_id)
            .instrument(instrument)
            .order_type(VenueOrderType::Limit)
            .side(MarketSide::Buy)
            .quantity(Decimal::from_f64(0.1).unwrap())
            .price(Some(Decimal::from_f64(50000.).unwrap()))
            .build()
            .unwrap();

        // Call place_order
        executor.place_order(order.clone()).await.unwrap();

        info!("Executor: {:?}", executor);

        // Assert that the order is in the executor's orders map
        assert!(executor.orders.contains_key(&order.id));
    }
}
