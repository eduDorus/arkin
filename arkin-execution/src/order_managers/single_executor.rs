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
pub struct SingleExecutorOrderManager {
    executor: Arc<dyn Executor>,
    portfolio: Arc<dyn Portfolio>,
    #[builder(default = DashMap::new())]
    execution_orders: DashMap<ExecutionOrderId, ExecutionOrder>,
    #[builder(default = DashMap::new())]
    venue_orders: DashMap<VenueOrderId, VenueOrder>,
}

#[async_trait]
impl OrderManager for SingleExecutorOrderManager {
    #[instrument(skip(self))]
    async fn start(&self, task_tracker: TaskTracker, shutdown: CancellationToken) -> Result<(), OrderManagerError> {
        info!("Starting order manager...");
        self.executor.start(task_tracker.clone(), shutdown.clone()).await?;
        info!("Order manager started");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cleanup(&self) -> Result<(), OrderManagerError> {
        info!("Cleaning up order manager...");
        self.executor.cleanup().await?;
        info!("Order manager cleaned up");
        Ok(())
    }

    #[instrument(skip(self))]
    async fn place_order(&self, order: ExecutionOrder) -> Result<(), OrderManagerError> {
        self.execution_orders.insert(order.id, order.clone());
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cancel_order(&self, id: ExecutionOrderId) -> Result<(), OrderManagerError> {
        let venue_order_ids = self
            .venue_orders
            .iter()
            .filter_map(|x| {
                let venue_order = x.value();
                if venue_order.execution_order_id == id {
                    Some(venue_order.id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        self.executor.cancel_orders(venue_order_ids).await?;
        self.execution_orders.remove(&id);
        Ok(())
    }

    #[instrument(skip(self))]
    async fn cancel_all_orders(&self) -> Result<(), OrderManagerError> {
        self.executor.cancel_all_orders().await?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn order_update(&self, fill: Fill) -> Result<(), OrderManagerError> {
        if let Some(mut order) = self.venue_orders.get_mut(&fill.venue_order_id) {
            order.add_fill(fill);
            Ok(())
        } else {
            Err(OrderManagerError::VenueOrderNotFound(fill.venue_order_id.to_string()))
        }
    }

    #[instrument(skip(self))]
    async fn order_status_update(&self, id: VenueOrderId, status: VenueOrderStatus) -> Result<(), OrderManagerError> {
        if let Some(mut order) = self.venue_orders.get_mut(&id) {
            order.update_status(status);
            Ok(())
        } else {
            Err(OrderManagerError::VenueOrderNotFound(id.to_string()))
        }
    }

    #[instrument(skip(self))]
    async fn position_update(&self, position: Position) -> Result<(), OrderManagerError> {
        self.portfolio.position_update(position).await?;
        Ok(())
    }

    #[instrument(skip(self))]
    async fn balance_update(&self, holding: Holding) -> Result<(), OrderManagerError> {
        self.portfolio.balance_update(holding).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockExecutor;
    use arkin_core::test_utils::*;
    use arkin_portfolio::MockPortfolio;
    use rust_decimal::prelude::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_place_order() {
        // Create mock Executor and Portfolio
        let mock_executor = MockExecutor::new();
        let mock_portfolio = MockPortfolio::new();

        // Build the SingleExecutorOrderManager with mocks
        let order_manager = SingleExecutorOrderManagerBuilder::default()
            .executor(Arc::new(mock_executor))
            .portfolio(Arc::new(mock_portfolio))
            .build()
            .unwrap();

        // Create a test ExecutionOrder
        let instrument = binance_btc_usdt_perp();
        let execution_order = ExecutionOrderBuilder::default()
            .instrument(instrument)
            .execution_type(ExecutionOrderStrategy::Market)
            .side(MarketSide::Buy)
            .quantity(Quantity::from_f64(1.0).unwrap())
            .build()
            .unwrap();

        // Call place_order
        order_manager.place_order(execution_order.clone()).await.unwrap();

        // Assert that the order is in the execution_orders map
        assert!(order_manager.execution_orders.contains_key(&execution_order.id));
    }
}
