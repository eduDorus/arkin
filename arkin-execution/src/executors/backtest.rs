use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use derive_builder::Builder;
use tokio_util::sync::CancellationToken;
use tracing::{info, instrument};

use arkin_core::prelude::*;

use crate::{Executor, ExecutorError};

#[derive(Debug, Builder)]
#[builder(setter(into))]
pub struct BacktestExecutor {
    pubsub: Arc<PubSub>,
    #[builder(default)]
    orders: DashMap<VenueOrderId, VenueOrder>,
}

impl BacktestExecutor {
    pub async fn ack_orders(&self) -> Vec<VenueOrderId> {
        // Change order state of new orders to open
        let mut acked_orders = vec![];
        self.orders.alter_all(|_, mut order| {
            if order.status == VenueOrderStatus::New {
                order.update_status(VenueOrderStatus::Placed);
                acked_orders.push(order.id.clone());
            }
            order
        });
        for order in acked_orders.iter() {
            info!("Order acked: {:?}", order);
        }
        acked_orders
    }

    pub async fn ack_cancels(&self) -> Vec<VenueOrderId> {
        // Change order state of cancelling orders to cancelled
        let mut acked_orders = vec![];
        self.orders.alter_all(|_, mut order| {
            if order.is_cancelling() {
                order.ack_cancel();
                acked_orders.push(order.id.clone());
            }
            order
        });
        for order in acked_orders.iter() {
            info!("Order cancelled: {:?}", order);
        }
        acked_orders
    }

    pub fn list_orders(&self) -> Vec<VenueOrder> {
        self.orders.iter().map(|order| order.value().clone()).collect()
    }

    pub fn list_open_orders(&self) -> Vec<VenueOrder> {
        self.orders
            .iter()
            .filter(|order| !order.value().is_active())
            .map(|order| order.value().clone())
            .collect()
    }

    pub fn list_finalized_orders(&self) -> Vec<VenueOrder> {
        self.orders
            .iter()
            .filter(|order| order.value().is_finalized())
            .map(|order| order.value().clone())
            .collect()
    }
}

#[async_trait]
impl Executor for BacktestExecutor {
    #[instrument(skip_all)]
    async fn start(&self, shutdown: CancellationToken) -> Result<(), ExecutorError> {
        info!("Starting simulation executor...");
        let mut venue_orders = self.pubsub.subscribe::<VenueOrder>();
        loop {
            tokio::select! {
                Ok(order) = venue_orders.recv() => {
                    info!("SimulationExecutor received order: {}", order.id);
                    self.orders.insert(order.id.clone(), order);
                }
                _ = shutdown.cancelled() => {
                    break;
                }
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    use rust_decimal::prelude::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_backtest_executor_place_order() {
        // Create executor
        let executor = BacktestExecutorBuilder::default().build().unwrap();

        // Create a sample VenueOrder
        let instrument = binance_btc_usdt_perp();
        let order = VenueOrderBuilder::default()
            .execution_order_id(ExecutionOrderId::new_v4())
            .instrument(instrument)
            .order_type(VenueOrderType::Limit)
            .side(MarketSide::Buy)
            .quantity(Decimal::from_f64(0.1).unwrap())
            .price(Some(Decimal::from_f64(50000.).unwrap()))
            .build()
            .unwrap();

        // Call place_order
        executor.place_order(order.clone()).await.unwrap();

        executor.ack_orders().await;

        // Get the list of orders
        let orders = executor
            .orders
            .iter()
            .map(|order| order.value().clone())
            .collect::<Vec<VenueOrder>>();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].status, VenueOrderStatus::Placed);
    }

    #[test(tokio::test)]
    async fn test_backtest_executor_cancel_order() {
        // Create executor
        let executor = BacktestExecutorBuilder::default().build().unwrap();

        // Create a sample VenueOrder
        let instrument = binance_btc_usdt_perp();
        let order = VenueOrderBuilder::default()
            .execution_order_id(ExecutionOrderId::new_v4())
            .instrument(instrument)
            .order_type(VenueOrderType::Limit)
            .side(MarketSide::Buy)
            .quantity(Decimal::from_f64(0.1).unwrap())
            .price(Some(Decimal::from_f64(50000.).unwrap()))
            .build()
            .unwrap();

        // Call place_order
        executor.place_order(order.clone()).await.unwrap();
        executor.ack_orders().await;
        executor.cancel_order(order.id).await.unwrap();
        executor.ack_cancels().await;

        let orders = executor.list_finalized_orders();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].status, VenueOrderStatus::Cancelled);
    }
}
