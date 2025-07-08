#![allow(dead_code)]
use std::sync::Arc;

use async_trait::async_trait;
use time::OffsetDateTime;
use tracing::{info, instrument, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{ExecutionOrderBook, VenueOrderBook};

#[derive(TypedBuilder)]
pub struct OrderManager {
    identifier: String,
    time: Arc<dyn SystemTime>,
    publisher: Arc<dyn Publisher>,
    execution_order_book: ExecutionOrderBook,
    venue_order_book: VenueOrderBook,
}

impl OrderManager {
    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn new_execution_order(&self, order: &ExecutionOrder) {
        info!(target: "order_manager", "received new execution order {}", order.id);

        // add to execution order book
        info!(target: "order_manager", "adding execution order {} to order book", order.id);
        self.execution_order_book.insert(order.clone());

        // match order.order_type {
        //     ExecutionOrderType::Taker => {
        //         info!(target: "order_manager", "received new execution order {}", order.id);
        //         let venue_order = VenueOrder::builder()
        //             .id(order.id)
        //             .strategy(order.strategy.clone())
        //             .instrument(order.instrument.clone())
        //             .side(order.side)
        //             .quantity(order.quantity)
        //             .price(order.price)
        //             .order_type(VenueOrderType::Market)
        //             .created_at(self.time.now().await)
        //             .updated_at(self.time.now().await)
        //             .build();
        //         self.publisher.publish(Event::NewVenueOrder(venue_order.into())).await;
        //     }
        //     ExecutionOrderType::Maker => {
        //         info!(target: "order_manager", "received new execution order {}", order.id);
        //         let venue_order = VenueOrder::builder()
        //             .id(order.id)
        //             .strategy(order.strategy.clone())
        //             .instrument(order.instrument.clone())
        //             .side(order.side)
        //             .quantity(order.quantity)
        //             .price(order.price)
        //             .order_type(VenueOrderType::Limit)
        //             .created_at(self.time.now().await)
        //             .updated_at(self.time.now().await)
        //             .build();
        //         self.publisher.publish(Event::NewVenueOrder(venue_order.into())).await;
        //     }
        //     _ => {
        //         warn!("Unsupported order type: {}", order);
        //     }
        // }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn cancel_execution_order(&self, order: &ExecutionOrder) {
        info!(target: "order_manager", "received cancel for execution order {}", order.id);
        let mut order = order.clone();
        order.update_status(ExecutionOrderStatus::Cancelled, self.time.now().await);

        info!(target: "order_manager", "updating order {} to cancelled in order book", order.id);
        self.execution_order_book.update(order);
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn cancel_all_execution_orders(&self, _time: &OffsetDateTime) {
        info!(target: "order_manager", "received cancel all execution orders");

        for order in self.execution_order_book.list_orders() {
            let mut order = order.clone();
            order.update_status(ExecutionOrderStatus::Cancelled, self.time.now().await);

            info!(target: "order_manager", "updating order {} to cancelled in order book", order.id);
            self.execution_order_book.update(order);
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_inflight(&self, order: &VenueOrder) {
        info!(target: "order_manager", "received status inflight for venue order {}", order.id);
        self.venue_order_book.update(order.clone());
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_placed(&self, order: &VenueOrder) {
        info!(target: "order_manager", "received status placed for venue order {}", order.id);
        self.venue_order_book.update(order.clone());
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_rejected(&self, order: &VenueOrder) {
        info!(target: "order_manager", "received status rejected for venue order {}", order.id);
        self.venue_order_book.update(order.clone());
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_fill(&self, order: &VenueOrder) {
        info!(target: "order_manager", "received fill for venue order {}", order.id);

        // Handle case where execution order ID is missing
        let exec_order_id = match order.execution_order_id {
            Some(id) => id,
            None => {
                warn!(target: "order_manager", "received fill from venue order {} without execution order id", order.id);
                return;
            }
        };

        // Retrieve execution order or log a warning if not found
        let mut exec_order = match self.execution_order_book.get(exec_order_id) {
            Some(order) => order,
            None => {
                warn!(target: "order_manager", "can't find execution order {} to add fill", exec_order_id);
                return;
            }
        };

        // Add fill details and update books
        exec_order.add_fill(
            order.updated_at,
            order.last_fill_price,
            order.last_fill_quantity,
            order.last_fill_commission,
        );
        self.execution_order_book.update(exec_order);
        self.venue_order_book.update(order.clone());
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_cancelled(&self, order: &VenueOrder) {
        info!(target: "order_manager", "received status cancelled for venue order {}", order.id);
        self.venue_order_book.update(order.clone());
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_expired(&self, order: &VenueOrder) {
        info!(target: "order_manager", "received status expired for venue order {}", order.id);
        self.venue_order_book.update(order.clone());
    }
}

#[async_trait]
impl Runnable for OrderManager {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            Event::NewExecutionOrder(eo) => self.new_execution_order(eo).await,
            Event::CancelExecutionOrder(eo) => self.cancel_execution_order(eo).await,
            Event::CancelAllExecutionOrders(t) => self.cancel_all_execution_orders(t).await,
            Event::VenueOrderInflight(vo) => self.venue_order_inflight(vo).await,
            Event::VenueOrderPlaced(vo) => self.venue_order_placed(vo).await,
            Event::VenueOrderRejected(vo) => self.venue_order_rejected(vo).await,
            Event::VenueOrderFill(vo) => self.venue_order_fill(vo).await,
            Event::VenueOrderCancelled(vo) => self.venue_order_cancelled(vo).await,
            Event::VenueOrderExpired(vo) => self.venue_order_expired(vo).await,
            e => warn!(target: "order_manager", "received unused event {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkin_core::test_utils::{MockPublisher, MockTime};
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    #[tokio::test]
    #[test_log::test]
    async fn test_new_execution_order() {
        // Setup: Initialize OrderManager with mock dependencies
        let time = MockTime::new();
        let publisher = MockPublisher::new();
        let execution_order_book = ExecutionOrderBook::default(); // Assumes default constructor
        let venue_order_book = VenueOrderBook::default();
        let order_manager = OrderManager::builder()
            .identifier("test".to_string())
            .time(time.clone())
            .publisher(publisher)
            .execution_order_book(execution_order_book)
            .venue_order_book(venue_order_book)
            .build();

        // Create a new execution order
        let order = ExecutionOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(ExecutionOrderType::Maker)
            .side(MarketSide::Buy)
            .price(dec!(0))
            .quantity(dec!(1))
            .status(ExecutionOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();

        // Execute: Handle the NewExecutionOrder event
        order_manager.handle_event(Event::NewExecutionOrder(order.clone().into())).await;

        // Verify: Check that the order is in the execution order book with status New
        let retrieved_order = order_manager
            .execution_order_book
            .get(order.id)
            .expect("Order should exist in the book");
        assert_eq!(retrieved_order.status, ExecutionOrderStatus::New, "Order status should be New");
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_venue_order_fill() {
        // Setup: Initialize OrderManager with mock dependencies
        let time = MockTime::new();
        let publisher = MockPublisher::new();
        let execution_order_book = ExecutionOrderBook::default();
        let venue_order_book = VenueOrderBook::default();
        let order_manager = OrderManager::builder()
            .identifier("test".to_string())
            .time(time.clone())
            .publisher(publisher)
            .execution_order_book(execution_order_book)
            .venue_order_book(venue_order_book)
            .build();

        // Create and insert an execution order
        let exec_order = ExecutionOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(ExecutionOrderType::Maker)
            .side(MarketSide::Buy)
            .price(dec!(0))
            .quantity(dec!(1))
            .status(ExecutionOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();

        order_manager.execution_order_book.insert(exec_order.clone());

        // Create and insert a venue order linked to the execution order
        let venue_order = VenueOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .execution_order_id(Some(exec_order.id))
            .order_type(VenueOrderType::Market)
            .side(MarketSide::Buy)
            .price(dec!(0))
            .quantity(dec!(1))
            .status(VenueOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();
        order_manager.venue_order_book.insert(venue_order.clone());

        // Create an updated venue order with fill details
        let mut filled_venue_order = venue_order.clone();
        filled_venue_order.status = VenueOrderStatus::PartiallyFilled;
        filled_venue_order.last_fill_price = dec!(100);
        filled_venue_order.last_fill_quantity = dec!(10);
        filled_venue_order.last_fill_commission = dec!(1);
        filled_venue_order.updated_at = time.now().await;

        // Execute: Handle the VenueOrderFill event
        order_manager
            .handle_event(Event::VenueOrderFill(filled_venue_order.clone().into()))
            .await;

        // Verify: Check the venue order in the book is updated
        let retrieved_venue_order = order_manager
            .venue_order_book
            .get(venue_order.id)
            .expect("Venue order should exist in the book");
        assert_eq!(
            retrieved_venue_order.status,
            VenueOrderStatus::PartiallyFilled,
            "Venue order status should be PartiallyFilled"
        );
        assert_eq!(
            retrieved_venue_order.last_fill_quantity,
            dec!(10),
            "Last fill quantity should match"
        );

        // Verify: Check the execution order has the fill added
        let retrieved_exec_order = order_manager
            .execution_order_book
            .get(exec_order.id)
            .expect("Execution order should exist in the book");
        assert_eq!(
            retrieved_exec_order.filled_quantity,
            dec!(10),
            "Execution order filled quantity should be updated"
        );
    }
}
