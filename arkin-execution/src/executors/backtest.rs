use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use derive_builder::Builder;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use tokio::select;
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
    #[builder(default = dec!(0.002))]
    taker_commission: Decimal,
    #[builder(default = dec!(0.001))]
    maker_commission: Decimal,
}

impl BacktestExecutor {
    pub fn list_open_orders(&self) -> Vec<VenueOrder> {
        self.orders
            .iter()
            .filter(|order| !order.value().is_active())
            .map(|order| order.value().clone())
            .collect()
    }

    pub fn fill_order(&self, id: VenueOrderId, fill: VenueOrderFill) {
        if let Some(mut order) = self.orders.get_mut(&id) {
            info!("Filling order: {:?}", fill);
            order.add_fill(fill);
        }

        // Remove the order if it is filled
        let is_finalized = self.orders.get(&id).map(|order| order.is_finalized()).unwrap_or(false);
        if is_finalized {
            self.orders.remove(&id);
        }
    }
}

#[async_trait]
impl Executor for BacktestExecutor {
    #[instrument(skip_all)]
    async fn start(&self, shutdown: CancellationToken) -> Result<(), ExecutorError> {
        info!("Starting simulation executor...");
        let mut tick_updates = self.pubsub.subscribe::<Tick>();
        let mut venue_orders = self.pubsub.subscribe::<VenueOrder>();
        loop {
            select! {
                Ok(order) = venue_orders.recv() => {
                    info!("SimulationExecutor received order: {}", order.id);
                    // Notify the order has been received
                    self.orders.insert(order.id.clone(), order.clone());
                    let update = VenueOrderStateBuilder::default()
                        .id(order.id.clone())
                        .status(VenueOrderStatus::Placed)
                        .build()
                        .unwrap();
                    info!("Order acked: {:?}", order.id);
                    self.pubsub.publish::<VenueOrderState>(update);

                }
                Ok(tick) = tick_updates.recv() => {
                    info!("SimulationExecutor received tick: {}", tick.instrument);
                    // Fill the order
                    let open_orders = self.list_open_orders();

                    // check if we got a price for the instrument
                    for order in open_orders.iter() {
                        if order.instrument == tick.instrument {
                            let price = if order.side == MarketSide::Buy {
                                tick.ask_price()
                            } else {
                                tick.bid_price()
                            };
                            let quantity = order.quantity;
                            let commission = match order.order_type {
                                VenueOrderType::Market => (price * quantity) * self.taker_commission,
                                VenueOrderType::Limit => (price * quantity) * self.maker_commission,
                            };
                            let fill = VenueOrderFillBuilder::default()
                                .id(order.id.clone())
                                .price(price)
                                .quantity(order.quantity)
                                .commission(commission)
                                .build()
                                .unwrap();

                            self.fill_order(order.id.clone(), fill.clone());
                            self.pubsub.publish::<VenueOrderFill>(fill);
                        }
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

    use rust_decimal_macros::dec;
    use test_log::test;
    use tokio_util::task::TaskTracker;

    #[test(tokio::test)]
    async fn test_backtest_executor_place_order() {
        // Create executor
        let pubsub = Arc::new(PubSub::new());
        let executor = Arc::new(BacktestExecutorBuilder::default().pubsub(pubsub.clone()).build().unwrap());

        // Start executor
        let tracker = TaskTracker::new();
        let shutdown = CancellationToken::new();
        let shutdown_clone = shutdown.clone();
        tracker.spawn(async move {
            executor.start(shutdown_clone).await.unwrap();
        });

        // Wait for executor to start
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // Create a sample VenueOrder
        let order = VenueOrderBuilder::default()
            .execution_order_id(ExecutionOrderId::new_v4())
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(VenueOrderType::Market)
            .side(MarketSide::Buy)
            .price(None)
            .quantity(Decimal::from_f64(0.1).unwrap())
            .build()
            .unwrap();

        // Subscribe to fill and updates
        let mut updates = pubsub.subscribe::<VenueOrderState>();
        let mut fills = pubsub.subscribe::<VenueOrderFill>();

        // Publish the order
        info!("Publishing order: {:?}", order);
        pubsub.publish::<VenueOrder>(order.clone());

        // Check for ack
        let ack = updates.recv().await.unwrap();
        assert_eq!(ack.status, VenueOrderStatus::Placed);

        // Send price update
        let tick = TickBuilder::default()
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(0 as u64)
            .bid_price(dec!(50000))
            .bid_quantity(dec!(1))
            .ask_price(dec!(50001))
            .ask_quantity(dec!(1))
            .build()
            .expect("Failed to build Tick");
        pubsub.publish::<Tick>(tick);

        // Check for fill
        let fill = fills.recv().await.unwrap();
        assert_eq!(fill.price, Decimal::from_f64(50001.).unwrap());
        assert_eq!(fill.quantity, Decimal::from_f64(0.1).unwrap());

        // Shutdown
        shutdown.cancel();
        tracker.close();
        tracker.wait().await;
    }
}
