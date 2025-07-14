use std::sync::Arc;

use arkin_core::prelude::*;
use async_trait::async_trait;
use time::UtcDateTime;
use tracing::{info, instrument, warn};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(TypedBuilder)]
pub struct ExecutionStrategy {
    identifier: String,
    #[builder(default = ExecutionStrategyType::Taker)]
    strategy_type: ExecutionStrategyType,
    time: Arc<dyn SystemTime>,
    publisher: Arc<dyn Publisher>,
    exec_order_book: Arc<ExecutionOrderBook>,
    venue_order_book: Arc<VenueOrderBook>,
}

impl ExecutionStrategy {
    pub fn new(
        identifier: &str,
        strategy_type: ExecutionStrategyType,
        time: Arc<dyn SystemTime>,
        publisher: Arc<dyn Publisher>,
        exec_order_book: Arc<ExecutionOrderBook>,
        venue_order_book: Arc<VenueOrderBook>,
    ) -> Arc<Self> {
        Self {
            identifier: identifier.to_owned(),
            strategy_type,
            time,
            publisher,
            exec_order_book,
            venue_order_book,
        }
        .into()
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn new_maker_execution_order(&self, exec_order: &ExecutionOrder) {
        info!(target: "exec_strategy", "received new execution order {}", exec_order.id);

        // Validate order strategy
        if exec_order.exec_strategy_type != self.strategy_type {
            warn!(target: "exec_strategy", "received wrong execution order type {}", exec_order.exec_strategy_type);
            return;
        }

        // add to execution order book
        let mut order = exec_order.clone();
        self.exec_order_book.insert(order.clone());
        order.update_status(ExecutionOrderStatus::Active, self.time.now().await);
        self.exec_order_book.update(order.clone());

        // Create market order
        let venue_order = VenueOrder::builder()
            .id(Uuid::new_v4())
            .execution_order_id(Some(order.id))
            .strategy(order.strategy.clone())
            .instrument(order.instrument.clone())
            .side(order.side)
            .quantity(order.quantity)
            .price(order.price)
            .order_type(VenueOrderType::Market)
            .created_at(self.time.now().await)
            .updated_at(self.time.now().await)
            .build();
        info!(target: "exec_strategy", "created new venue order {}", venue_order.id);

        // Add to the order book
        self.venue_order_book.insert(venue_order.clone());

        // Publish the new venue order
        self.publisher.publish(Event::NewVenueOrder(venue_order.clone().into())).await;
        info!(target: "exec_strategy", "published new venue order {}", venue_order.id);
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn cancel_maker_execution_order(&self, order: &ExecutionOrder) {
        info!(target: "exec_strategy", "received cancel for execution order {}", order.id);

        // Update the exec order book
        let mut order = order.clone();
        order.update_status(ExecutionOrderStatus::Cancelling, self.time.now().await);
        self.exec_order_book.update(order.clone());

        // Cancel all venue orders linked to the exec order
        let venue_orders = self.venue_order_book.list_orders_by_exec_id(order.id);
        for venue_order in venue_orders {
            self.publisher
                .publish(Event::CancelVenueOrder(venue_order.clone().into()))
                .await;
            info!(target: "exec_strategy", "send cancel order for venue order {}", venue_order.id);
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn cancel_all_maker_execution_orders(&self, _time: &UtcDateTime) {
        info!(target: "exec_strategy", "received cancel all execution orders");

        // Change all exec orders to cancelling
        for exec_order in self.exec_order_book.list_orders_by_exec_strategy(self.strategy_type) {
            let venue_orders = self.venue_order_book.list_orders_by_exec_id(exec_order.id);

            let mut order = exec_order.clone();
            if venue_orders.is_empty() {
                order.update_status(ExecutionOrderStatus::Cancelled, self.time.now().await);
                self.exec_order_book.update(order.clone());
            } else {
                order.update_status(ExecutionOrderStatus::Cancelling, self.time.now().await);
                self.exec_order_book.update(order.clone());
                for venue_order in venue_orders {
                    self.publisher
                        .publish(Event::CancelVenueOrder(venue_order.clone().into()))
                        .await;
                    info!(target: "exec_strategy", "send cancel order for venue order {}", venue_order.id);
                }
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_inflight(&self, order: &VenueOrder) {
        info!(target: "exec_strategy", "received status inflight for venue order {}", order.id);

        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.update(order.clone());
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_placed(&self, order: &VenueOrder) {
        info!(target: "exec_strategy", "received status placed for venue order {}", order.id);

        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.update(order.clone());
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_rejected(&self, order: &VenueOrder) {
        info!(target: "exec_strategy", "received status rejected for venue order {}", order.id);

        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.update(order.clone());
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_fill(&self, order: &VenueOrder) {
        info!(target: "exec_strategy", "received fill for venue order {}", order.id);

        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.update(order.clone());

                // Add fill details and update books
                let mut exec_order = self.exec_order_book.get(id).unwrap();
                exec_order.add_fill(
                    order.updated_at,
                    order.last_fill_price,
                    order.last_fill_quantity,
                    order.last_fill_commission,
                );
                self.exec_order_book.update(exec_order.clone());
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_cancelled(&self, order: &VenueOrder) {
        info!(target: "exec_strategy", "received status cancelled for venue order {}", order.id);
        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.update(order.clone());
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_expired(&self, order: &VenueOrder) {
        info!(target: "exec_strategy", "received status expired for venue order {}", order.id);
        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.update(order.clone());
            }
        }
    }
}

#[async_trait]
impl Runnable for ExecutionStrategy {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            Event::NewMakerExecutionOrder(eo) => self.new_maker_execution_order(eo).await,
            Event::CancelMakerExecutionOrder(eo) => self.cancel_maker_execution_order(eo).await,
            Event::CancelAllMakerExecutionOrders(t) => self.cancel_all_maker_execution_orders(t).await,
            Event::VenueOrderInflight(vo) => self.venue_order_inflight(vo).await,
            Event::VenueOrderPlaced(vo) => self.venue_order_placed(vo).await,
            Event::VenueOrderRejected(vo) => self.venue_order_rejected(vo).await,
            Event::VenueOrderFill(vo) => self.venue_order_fill(vo).await,
            Event::VenueOrderCancelled(vo) => self.venue_order_cancelled(vo).await,
            Event::VenueOrderExpired(vo) => self.venue_order_expired(vo).await,
            e => warn!(target: "exec_strategy", "received unused event {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkin_core::test_utils::{MockPublisher, MockTime};
    use rust_decimal::prelude::*;
    use uuid::Uuid;

    #[tokio::test]
    #[test_log::test]
    async fn test_new_execution_order() {
        // Setup: Initialize OrderManager with mock dependencies
        let time = MockTime::new();
        let publisher = MockPublisher::new();
        let execution_order_book = ExecutionOrderBook::new(true);
        let venue_order_book = VenueOrderBook::new(true);
        let exec_strategy = ExecutionStrategy::builder()
            .identifier("test".to_string())
            .time(time.to_owned())
            .publisher(publisher.to_owned())
            .exec_order_book(execution_order_book.to_owned())
            .venue_order_book(venue_order_book.to_owned())
            .build();

        // Create a new execution order
        let order_1 = ExecutionOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .exec_strategy_type(ExecutionStrategyType::Taker)
            .side(MarketSide::Buy)
            .price(dec!(0))
            .quantity(dec!(1))
            .status(ExecutionOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();

        // Execute: Handle the NewExecutionOrder event
        exec_strategy
            .handle_event(Event::NewMakerExecutionOrder(order_1.clone().into()))
            .await;

        // Verify: Check that the order is in the execution order book with status New
        let retrieved_order = exec_strategy
            .exec_order_book
            .get(order_1.id)
            .expect("Order should exist in the book");
        assert_eq!(
            retrieved_order.status,
            ExecutionOrderStatus::Active,
            "Order status should be New"
        );
        assert_eq!(1, execution_order_book.len(), "order book should have 1 order");

        // Create a new execution order
        let order_2 = ExecutionOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .exec_strategy_type(ExecutionStrategyType::Maker)
            .side(MarketSide::Sell)
            .price(dec!(0))
            .quantity(dec!(1))
            .status(ExecutionOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();

        // Execute: Handle the NewExecutionOrder event
        exec_strategy
            .handle_event(Event::NewMakerExecutionOrder(order_2.clone().into()))
            .await;

        // Verify: Check that the order is in the execution order book with status New
        let retrieved_order = exec_strategy.exec_order_book.get(order_2.id);
        assert_eq!(retrieved_order, None);
        // // .expect("Order should exist in the book");
        // assert_eq!(
        //     retrieved_order.status,
        //     ExecutionOrderStatus::Active,
        //     "Order status should be New"
        // );
        assert_eq!(1, execution_order_book.len(), "order book should have 1 order");

        // Get venue order and fill it
        let venue_orders = venue_order_book.list_orders();
        for mut order in venue_orders {
            order.add_fill(time.now().await, dec!(100), dec!(1), dec!(0.05));
            exec_strategy.handle_event(Event::VenueOrderFill(order.into())).await;
        }

        assert_eq!(0, execution_order_book.len(), "order book should have o orders (autoclean)");
        assert_eq!(1, publisher.get_events().await.len(), "expect 1 event (new venue order)")
    }
}
