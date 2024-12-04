use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;
use uuid::Uuid;

use crate::{Executor, ExecutorError};

#[derive(Debug, TypedBuilder)]

pub struct SimulationExecutor {
    pubsub: Arc<PubSub>,
    #[builder(default)]
    orders: DashMap<VenueOrderId, Arc<VenueOrder>>,
    #[builder(default = dec!(0.0005))]
    taker_commission: Decimal,
    #[builder(default = dec!(0.0002))]
    maker_commission: Decimal,
    #[builder(default = DashMap::new())]
    balances: DashMap<Arc<Asset>, Holding>,
}

impl SimulationExecutor {
    pub fn list_open_orders(&self) -> Vec<Arc<VenueOrder>> {
        self.orders
            .iter()
            .filter(|order| !order.value().is_active())
            .map(|order| order.value().clone())
            .collect()
    }

    pub fn fill_order(&self, _id: VenueOrderId, _fill: Arc<VenueOrderFill>) {
        unimplemented!("SimulationExecutor::fill_order")
        // if let Some(mut order) = self.orders.get_mut(&id) {
        //     order.add_fill(fill.clone());
        //     info!("SimulationExecutor filled order: {}", fill);
        // }

        // // Remove the order if it is filled
        // let is_finalized = self.orders.get(&id).map(|order| order.is_finalized()).unwrap_or(false);
        // if is_finalized {
        //     self.orders.remove(&id);
        // }
    }

    pub fn update_balance(&self, asset: &Arc<Asset>, quantity: Decimal) {
        let mut entry = self.balances.entry(asset.clone()).or_insert(
            Holding::builder()
                .id(Uuid::new_v4())
                .asset(asset.clone())
                .balance(dec!(0))
                .build(),
        );
        entry.balance += quantity;
    }

    pub fn get_balance(&self, asset: &Arc<Asset>) -> Option<Holding> {
        self.balances.get(asset).map(|holding| holding.value().clone())
    }
}

#[async_trait]
impl Executor for SimulationExecutor {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), ExecutorError> {
        info!("Starting simulation executor...");
        // TODO: Send current balance
        let holding = Arc::new(
            Holding::builder()
                .id(Uuid::new_v4())
                .asset(test_usdt_asset())
                .balance(dec!(10000))
                .build(),
        );
        self.update_balance(&holding.asset, holding.balance);
        info!("Sending initial balance: {}", holding);
        self.pubsub.publish::<Holding>(holding);

        let mut tick_updates = self.pubsub.subscribe::<Tick>();
        let mut venue_orders = self.pubsub.subscribe::<VenueOrder>();
        loop {
            select! {
                Ok(order) = venue_orders.recv() => {
                    info!("SimulationExecutor received order: {}", order);
                    // Check if the order is valid and we have enough balance
                    // TODO: Check if the order is valid

                    // Notify the order has been received
                    self.orders.insert(order.id.clone(), order.clone());
                    let update = VenueOrderState::builder()
                        .id(order.id.clone())
                        .status(VenueOrderStatus::Placed)
                        .build();
                    info!("SimulationExecutor placed order: {}", order);
                    self.pubsub.publish::<VenueOrderState>(update.into());

                }
                Ok(tick) = tick_updates.recv() => {
                    debug!("SimulationExecutor received tick: {}", tick.instrument);
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

                            // Calculate commission
                            let mut commission = match order.order_type {
                                VenueOrderType::Market => (price * quantity) * self.taker_commission,
                                VenueOrderType::Limit => (price * quantity) * self.maker_commission,
                                _ => unimplemented!("Unsupported order type"),
                            };
                            commission = commission.round_dp(order.instrument.price_precision);

                            // Create the fill
                            let fill = VenueOrderFill::builder()
                                .venue_order(order.clone())
                                .instrument(order.instrument.clone())
                                .side(order.side.clone())
                                .price(price)
                                .quantity(order.quantity)
                                .commission(commission)
                                .build();
                            let fill = Arc::new(fill);


                            // Subtract the value from the balance
                            // self.update_balance(&order.instrument.base_asset, fill.market_value() + fill.commission);
                            self.fill_order(order.id.clone(), fill.clone());

                            // Publish
                            // info!("Publishing new balance: {}", holding);
                            // self.pubsub.publish::<Holding>(holding);
                            info!("SimulationExecutor publishing order filled: {}", order);
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

    async fn place_order(&self, order: Arc<VenueOrder>) -> Result<(), ExecutorError> {
        self.orders.insert(order.id, order.clone());
        info!("SimulationExecution placed order: {}", order);
        Ok(())
    }

    async fn place_orders(&self, orders: Vec<Arc<VenueOrder>>) -> Result<(), ExecutorError> {
        for order in orders {
            self.orders.insert(order.id, order);
        }
        Ok(())
    }

    async fn modify_order(&self, _order: Arc<VenueOrder>) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::modify_order")
    }

    async fn modify_orders(&self, _orders: Vec<Arc<VenueOrder>>) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::modify_orders")
    }

    async fn cancel_order(&self, _id: VenueOrderId) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::cancel_order")
        // if let Some(mut order) = self.orders.get_mut(&id) {
        //     order.cancel();
        //     info!("SimulationExecution cancelled order: {}", *order);
        //     Ok(())
        // } else {
        //     return Err(ExecutorError::InvalidOrder(id.to_string()));
        // }
    }

    async fn cancel_orders(&self, ids: Vec<VenueOrderId>) -> Result<(), ExecutorError> {
        for id in ids {
            self.cancel_order(id).await?;
        }
        Ok(())
    }

    async fn cancel_all_orders(&self) -> Result<(), ExecutorError> {
        unimplemented!("SimulationExecutor::cancel_all_orders")
        // for mut order in self.orders.iter_mut() {
        //     order.cancel();
        //     info!("SimulationExecution cancelled order: {}", *order);
        // }
        // Ok(())
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
        let executor = Arc::new(SimulationExecutor::builder().pubsub(pubsub.clone()).build());

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
        let order: Arc<VenueOrder> = VenueOrder::builder()
            .id(Uuid::new_v4())
            .portfolio(test_portfolio())
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(VenueOrderType::Market)
            .side(MarketSide::Buy)
            .price(None)
            .quantity(dec!(0.1))
            .build()
            .into();

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
        let tick = Arc::new(
            Tick::builder()
                .instrument(test_inst_binance_btc_usdt_perp())
                .tick_id(0 as u64)
                .bid_price(dec!(50000))
                .bid_quantity(dec!(1))
                .ask_price(dec!(50001))
                .ask_quantity(dec!(1))
                .build(),
        );
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
