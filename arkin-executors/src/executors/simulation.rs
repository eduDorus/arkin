use std::{collections::HashMap, sync::Arc, vec};

use async_trait::async_trait;
use rust_decimal::prelude::*;
use rust_decimal_macros::dec;
use tokio::{select, sync::RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{
    traits::{Executor, ExecutorService},
    ExecutorError,
};

#[derive(Debug, TypedBuilder)]
pub struct SimulationExecutor {
    pubsub: Arc<PubSub>,
    #[builder(default)]
    orders: RwLock<HashMap<VenueOrderId, VenueOrder>>,
    #[builder(default = dec!(0.0005))]
    taker_commission: Decimal,
    #[builder(default = dec!(0.0002))]
    maker_commission: Decimal,
}

impl SimulationExecutor {
    pub async fn execute_order(&self, order: &VenueOrder, tick: &Arc<Tick>) {
        info!("SimulationExecutor execute_order: {}", order);
        let price = if order.side == MarketSide::Buy {
            tick.ask_price()
        } else {
            tick.bid_price()
        };
        let quantity = order.quantity;

        // Calculate commission
        let commission = match order.order_type {
            VenueOrderType::Market => (price * quantity) * self.taker_commission,
            VenueOrderType::Limit => (price * quantity) * self.maker_commission,
            _ => unimplemented!("Unsupported order type"),
        };

        let mut order = order.clone();
        order.add_fill(tick.event_time, price, quantity, commission);
        self.pubsub.publish(Event::VenueOrderFillUpdate(order.clone().into())).await;

        // If the order is partially filled we need to update the hashmap else remove it
        if order.status != VenueOrderStatus::Filled {
            let mut orders = self.orders.write().await;
            orders.insert(order.id.clone(), order.clone());
        } else {
            let mut orders = self.orders.write().await;
            orders.remove(&order.id);
        }
    }

    pub async fn tick_update(&self, tick: Arc<Tick>) {
        // Check if orders is empty
        let lock = self.orders.read().await;
        if lock.is_empty() {
            return;
        }
        let orders = lock.clone();
        drop(lock);

        // check if we got a price for the instrument
        for (_id, order) in orders.iter() {
            if order.instrument == tick.instrument {
                // Execute market order at tob if limit order check if we can execute
                match order.order_type {
                    VenueOrderType::Market => {
                        info!("SimulationExecutor found market order: {}", order);
                        self.execute_order(order, &tick).await;
                    }
                    VenueOrderType::Limit => {
                        if order.side == MarketSide::Buy && tick.ask_price() <= order.price {
                            self.execute_order(order, &tick).await;
                        } else if order.side == MarketSide::Sell && tick.bid_price() >= order.price {
                            self.execute_order(order, &tick).await;
                        }
                    }
                    _ => unimplemented!("Unsupported order type"),
                };
            }
        }
    }
}

#[async_trait]
impl Executor for SimulationExecutor {
    async fn get_balances(&self, _portfolio: &Arc<Portfolio>) -> Result<Vec<Arc<Balance>>, ExecutorError> {
        Ok(vec![])
    }

    async fn get_positions(&self, _portfolio: &Arc<Portfolio>) -> Result<Vec<Arc<Position>>, ExecutorError> {
        Ok(vec![])
    }

    async fn place_order(&self, order: Arc<VenueOrder>) -> Result<(), ExecutorError> {
        debug!("SimulationExecutor placing order: {}", order.id);
        let mut orders = self.orders.write().await;

        if !orders.contains_key(&order.id) {
            let mut order = (*order).clone();
            order.update_status(VenueOrderStatus::Placed);
            orders.insert(order.id.clone(), order.clone());
            self.pubsub.publish(Event::VenueOrderUpdate(order.into())).await;
        } else {
            warn!("Order already exists: {}", order);
        }
        Ok(())
    }

    async fn cancel_order(&self, id: VenueOrderId) -> Result<(), ExecutorError> {
        let mut orders = self.orders.write().await;

        if let Some(mut order) = orders.remove(&id) {
            order.cancel();
            self.pubsub.publish(Event::VenueOrderUpdate(order.into())).await;
        } else {
            warn!("Order not found: {}", id);
        }
        Ok(())
    }

    async fn cancel_orders_by_instrument(&self, instrument: Arc<Instrument>) -> Result<(), ExecutorError> {
        let mut orders = self.orders.write().await;
        let mut to_cancel = vec![];

        for (id, order) in orders.iter() {
            if order.instrument == instrument {
                to_cancel.push(id.clone());
            }
        }
        for id in to_cancel {
            if let Some(mut order) = orders.remove(&id) {
                order.cancel();
                self.pubsub.publish(Event::VenueOrderUpdate(order.into())).await;
            }
        }
        Ok(())
    }

    async fn cancel_all_orders(&self) -> Result<(), ExecutorError> {
        let mut orders = self.orders.write().await;

        for (_id, mut order) in orders.drain() {
            order.cancel();
            self.pubsub.publish(Event::VenueOrderUpdate(order.into())).await;
        }
        Ok(())
    }
}

#[async_trait]
impl RunnableService for SimulationExecutor {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting simulation executor...");

        let mut rx = self.pubsub.subscribe();
        loop {
            select! {
                Ok(event) = rx.recv() => {
                    match event {
                       Event::VenueOrderNew(order) => {
                           debug!("SimulationExecutor received order: {}", order);
                           self.place_order(order).await?;
                       }
                       Event::VenueOrderCancel(id) => {
                           debug!("SimulationExecutor received order cancel: {}", id);
                           self.cancel_order(id).await?;
                       }
                       Event::Tick(tick) => {
                           debug!("SimulationExecutor received tick: {}", tick.instrument);
                           self.tick_update(tick).await;
                       }
                       _ => {}

                    }
                }
                _ = shutdown.cancelled() => {
                    info!("SimulationExecutor shutdown...");
                    break;
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl ExecutorService for SimulationExecutor {}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::sync::Arc;
    use test_log::test;
    use time::OffsetDateTime;
    use tokio::time::{timeout, Duration};

    #[test(tokio::test)]
    async fn test_place_order() {
        // Create a shared pubsub and executor.
        let pubsub = Arc::new(PubSub::new(1024));
        let executor = SimulationExecutor::builder().pubsub(Arc::clone(&pubsub)).build();

        // Create dummy portfolio and instrument.
        let strategy = test_strategy();
        let instrument = test_inst_binance_btc_usdt_perp();

        // Build an order.
        let order = VenueOrder::builder()
            .event_time(OffsetDateTime::now_utc())
            .strategy(Arc::clone(&strategy))
            .instrument(Arc::clone(&instrument))
            .side(MarketSide::Buy)
            .price(dec!(100))
            .quantity(dec!(1))
            .build();
        let order_id = order.id.clone();
        let order_arc = Arc::new(order);

        // Subscribe to pubsub events.
        let mut rx = pubsub.subscribe();

        // Place the order.
        executor.place_order(Arc::clone(&order_arc)).await.unwrap();

        // Verify the order was inserted and its status updated to Placed.
        {
            let orders = executor.orders.read().await;
            assert!(orders.contains_key(&order_id));
            let stored_order = orders.get(&order_id).unwrap();
            assert_eq!(stored_order.status, VenueOrderStatus::Placed);
        }

        // Verify that a VenueOrderUpdate event was published.
        // We use a timeout in case no event is published.
        let event = timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("Expected event within 1 sec")
            .expect("Event channel closed");
        match event {
            Event::VenueOrderUpdate(updated_order) => {
                assert_eq!(updated_order.id, order_id);
                assert_eq!(updated_order.status, VenueOrderStatus::Placed);
            }
            _ => panic!("Expected VenueOrderUpdate event"),
        }
    }

    #[test(tokio::test)]
    async fn test_cancel_order() {
        let pubsub = Arc::new(PubSub::new(1024));
        let executor = SimulationExecutor::builder().pubsub(Arc::clone(&pubsub)).build();

        let strategy = test_strategy();
        let instrument = test_inst_binance_btc_usdt_perp();

        let order = VenueOrder::builder()
            .event_time(OffsetDateTime::now_utc())
            .strategy(Arc::clone(&strategy))
            .instrument(Arc::clone(&instrument))
            .side(MarketSide::Sell)
            .price(dec!(200))
            .quantity(dec!(2))
            .build();
        let order_id = order.id.clone();

        executor.place_order(order.into()).await.unwrap();

        let mut rx = pubsub.subscribe();
        executor.cancel_order(order_id.clone()).await.unwrap();

        // Check that the order was removed.
        {
            let orders = executor.orders.read().await;
            assert!(!orders.contains_key(&order_id));
        }

        // Verify that a VenueOrderUpdate event was published with a cancelled status.
        let event = timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("Expected event within 1 sec")
            .expect("Event channel closed");
        match event {
            Event::VenueOrderUpdate(updated_order) => {
                assert_eq!(updated_order.id, order_id);
                // Assuming that order.cancel() sets the status to Cancelled.
                assert_eq!(updated_order.status, VenueOrderStatus::Cancelled);
            }
            _ => panic!("Expected VenueOrderUpdate event"),
        }
    }

    #[test(tokio::test)]
    async fn test_cancel_orders_by_instrument() {
        let pubsub = Arc::new(PubSub::new(1024));
        let executor = SimulationExecutor::builder().pubsub(Arc::clone(&pubsub)).build();

        let strategy = test_strategy();
        let instrument_btc = test_inst_binance_btc_usdt_perp();
        let instrument_eth = test_inst_binance_eth_usdt_perp();

        // Create two orders with the same instrument...
        let order1 = VenueOrder::builder()
            .event_time(OffsetDateTime::now_utc())
            .strategy(Arc::clone(&strategy))
            .instrument(Arc::clone(&instrument_btc))
            .side(MarketSide::Buy)
            .price(dec!(150))
            .quantity(dec!(3))
            .build();
        let order2 = VenueOrder::builder()
            .event_time(OffsetDateTime::now_utc())
            .strategy(Arc::clone(&strategy))
            .instrument(Arc::clone(&instrument_btc))
            .side(MarketSide::Sell)
            .price(dec!(155))
            .quantity(dec!(1))
            .build();
        // ...and one order with a different instrument.
        let order3 = VenueOrder::builder()
            .event_time(OffsetDateTime::now_utc())
            .strategy(Arc::clone(&strategy))
            .instrument(Arc::clone(&instrument_eth))
            .side(MarketSide::Buy)
            .price(dec!(160))
            .quantity(dec!(2))
            .build();

        {
            let mut orders = executor.orders.write().await;
            orders.insert(order1.id.clone(), order1.clone());
            orders.insert(order2.id.clone(), order2.clone());
            orders.insert(order3.id.clone(), order3.clone());
        }

        let mut rx = pubsub.subscribe();
        executor.cancel_orders_by_instrument(Arc::clone(&instrument_btc)).await.unwrap();

        // Orders with the given instrument should be removed.
        {
            let orders = executor.orders.read().await;
            assert!(!orders.contains_key(&order1.id));
            assert!(!orders.contains_key(&order2.id));
            // Order with a different instrument remains.
            assert!(orders.contains_key(&order3.id));
        }

        // Expect two cancellation events.
        let mut cancelled_ids = vec![];
        for _ in 0..2 {
            let event = timeout(Duration::from_secs(1), rx.recv())
                .await
                .expect("Expected event within 1 sec")
                .expect("Event channel closed");
            match event {
                Event::VenueOrderUpdate(updated_order) => {
                    cancelled_ids.push(updated_order.id);
                }
                _ => panic!("Expected VenueOrderUpdate event"),
            }
        }
        assert!(cancelled_ids.contains(&order1.id));
        assert!(cancelled_ids.contains(&order2.id));
    }

    #[test(tokio::test)]
    async fn test_cancel_all_orders() {
        let pubsub = Arc::new(PubSub::new(1024));
        let executor = SimulationExecutor::builder().pubsub(Arc::clone(&pubsub)).build();

        let strategy = test_strategy();
        let instrument = test_inst_binance_btc_usdt_perp();

        let order1 = VenueOrder::builder()
            .event_time(OffsetDateTime::now_utc())
            .strategy(Arc::clone(&strategy))
            .instrument(Arc::clone(&instrument))
            .side(MarketSide::Buy)
            .price(dec!(100))
            .quantity(dec!(1))
            .build();
        let order2 = VenueOrder::builder()
            .event_time(OffsetDateTime::now_utc())
            .strategy(Arc::clone(&strategy))
            .instrument(Arc::clone(&instrument))
            .side(MarketSide::Sell)
            .price(dec!(110))
            .quantity(dec!(2))
            .build();

        {
            let mut orders = executor.orders.write().await;
            orders.insert(order1.id.clone(), order1.clone());
            orders.insert(order2.id.clone(), order2.clone());
        }

        let mut rx = pubsub.subscribe();
        executor.cancel_all_orders().await.unwrap();

        // The orders map should now be empty.
        {
            let orders = executor.orders.read().await;
            assert!(orders.is_empty());
        }

        // Expect two cancellation events.
        let mut cancelled_ids = vec![];
        for _ in 0..2 {
            let event = timeout(Duration::from_secs(1), rx.recv())
                .await
                .expect("Expected event within 1 sec")
                .expect("Event channel closed");
            match event {
                Event::VenueOrderUpdate(updated_order) => {
                    cancelled_ids.push(updated_order.id);
                }
                _ => panic!("Expected VenueOrderUpdate event"),
            }
        }
        assert!(cancelled_ids.contains(&order1.id));
        assert!(cancelled_ids.contains(&order2.id));
    }

    #[test(tokio::test)]
    async fn test_market_order_execution() {
        let pubsub = Arc::new(PubSub::new(1024));
        let executor = SimulationExecutor::builder().pubsub(Arc::clone(&pubsub)).build();

        let strategy = test_strategy();
        let instrument = test_inst_binance_btc_usdt_perp();

        // Build a market buy order.
        let buy_order = VenueOrder::builder()
            .event_time(OffsetDateTime::now_utc())
            .strategy(Arc::clone(&strategy))
            .instrument(Arc::clone(&instrument))
            .side(MarketSide::Buy)
            .order_type(VenueOrderType::Market)
            .price(dec!(100)) // Price may be ignored for market orders.
            .quantity(dec!(1))
            .build();
        let buy_order_id = buy_order.id.clone();

        // Build a market sell order.
        let sell_order = VenueOrder::builder()
            .event_time(OffsetDateTime::now_utc())
            .strategy(Arc::clone(&strategy))
            .instrument(Arc::clone(&instrument))
            .side(MarketSide::Sell)
            .order_type(VenueOrderType::Market)
            .price(dec!(100))
            .quantity(dec!(1))
            .build();
        let sell_order_id = sell_order.id.clone();

        // Insert orders into the executor.
        info!("Placing buy order: {}", buy_order_id);
        executor.place_order(buy_order.into()).await.unwrap();
        info!("Placing sell order: {}", sell_order_id);
        executor.place_order(sell_order.into()).await.unwrap();

        let mut rx = pubsub.subscribe();

        // Create a tick where:
        // - For buy orders, execution price should be the ask price (e.g., 105).
        // - For sell orders, execution price should be the bid price (e.g., 95).
        let tick = Tick::builder()
            .event_time(OffsetDateTime::now_utc())
            .instrument(Arc::clone(&instrument))
            .tick_id(1)
            .ask_price(dec!(105))
            .ask_quantity(dec!(1))
            .bid_price(dec!(95))
            .bid_quantity(dec!(1))
            .build();
        let tick_arc = Arc::new(tick);

        info!("Updating tick");
        executor.tick_update(Arc::clone(&tick_arc)).await;

        // Collect two events (one for each order).
        info!("Checking");
        let mut events_received = 0;
        let mut updated_buy_order: Option<Arc<VenueOrder>> = None;
        let mut updated_sell_order: Option<Arc<VenueOrder>> = None;
        while events_received < 2 {
            info!("Waiting for event...");
            let event = rx.recv().await.expect("Channel closed");
            if let Event::VenueOrderUpdate(updated_order) = event {
                if updated_order.id == buy_order_id {
                    updated_buy_order = Some(updated_order);
                } else if updated_order.id == sell_order_id {
                    updated_sell_order = Some(updated_order);
                }
                events_received += 1;
            }
        }

        let executed_buy = updated_buy_order.expect("No update for buy order");
        let executed_sell = updated_sell_order.expect("No update for sell order");

        // Verify execution prices.
        assert_eq!(executed_buy.last_fill_price, dec!(105));
        assert_eq!(executed_sell.last_fill_price, dec!(95));
    }

    #[test(tokio::test)]
    async fn test_limit_order_execution() {
        let pubsub = Arc::new(PubSub::new(1024));
        let executor = SimulationExecutor::builder().pubsub(Arc::clone(&pubsub)).build();

        let strategy = test_strategy();
        let instrument = test_inst_binance_btc_usdt_perp();

        // Build a limit buy order with a limit price of 105.
        let buy_order = VenueOrder::builder()
            .event_time(OffsetDateTime::now_utc())
            .strategy(Arc::clone(&strategy))
            .instrument(Arc::clone(&instrument))
            .side(MarketSide::Buy)
            .order_type(VenueOrderType::Limit)
            .price(dec!(105))
            .quantity(dec!(1))
            .build();
        let buy_order_id = buy_order.id.clone();

        // Build a limit sell order with a limit price of 95.
        let sell_order = VenueOrder::builder()
            .event_time(OffsetDateTime::now_utc())
            .strategy(Arc::clone(&strategy))
            .instrument(Arc::clone(&instrument))
            .side(MarketSide::Sell)
            .order_type(VenueOrderType::Limit)
            .price(dec!(95))
            .quantity(dec!(1))
            .build();
        let sell_order_id = sell_order.id.clone();

        executor.place_order(buy_order.into()).await.unwrap();
        executor.place_order(sell_order.into()).await.unwrap();

        let mut rx = pubsub.subscribe();

        // --- First tick: conditions not met ---
        // For a buy limit order, tick.ask_price must be <= order.price.
        // Provide a tick with ask = 110 (> 105) and bid = 90 (< 95) for the sell order.
        let tick = Tick::builder()
            .event_time(OffsetDateTime::now_utc())
            .instrument(Arc::clone(&instrument))
            .tick_id(1)
            .ask_price(dec!(110))
            .ask_quantity(dec!(1))
            .bid_price(dec!(90))
            .bid_quantity(dec!(1))
            .build();
        let tick_arc = Arc::new(tick);
        executor.tick_update(Arc::clone(&tick_arc)).await;

        // Expect no execution events on tick1.
        // if let Ok(_) = timeout(Duration::from_secs(1), rx.recv()).await {
        //     panic!("No execution should occur when limit conditions are not met");
        // }

        // --- Second tick: conditions met ---
        // For the buy limit, use ask = 100 (<= 105).
        // For the sell limit, use bid = 100 (>= 95).
        let tick = Tick::builder()
            .event_time(OffsetDateTime::now_utc())
            .instrument(Arc::clone(&instrument))
            .tick_id(1)
            .ask_price(dec!(100))
            .ask_quantity(dec!(1))
            .bid_price(dec!(100))
            .bid_quantity(dec!(1))
            .build();
        let tick_arc = Arc::new(tick);
        executor.tick_update(Arc::clone(&tick_arc)).await;

        let mut events_received = 0;
        let mut updated_buy_order: Option<Arc<VenueOrder>> = None;
        let mut updated_sell_order: Option<Arc<VenueOrder>> = None;
        while events_received < 2 {
            let event = timeout(Duration::from_secs(1), rx.recv())
                .await
                .expect("Timeout waiting for event")
                .expect("Channel closed");
            if let Event::VenueOrderUpdate(updated_order) = event {
                if updated_order.id == buy_order_id {
                    updated_buy_order = Some(updated_order);
                } else if updated_order.id == sell_order_id {
                    updated_sell_order = Some(updated_order);
                }
                events_received += 1;
            }
        }

        let executed_buy = updated_buy_order.expect("No update for buy limit order");
        let executed_sell = updated_sell_order.expect("No update for sell limit order");

        // Verify that execution prices reflect the tick values.
        assert_eq!(executed_buy.last_fill_price, dec!(100));
        assert_eq!(executed_sell.last_fill_price, dec!(100));
    }
}
