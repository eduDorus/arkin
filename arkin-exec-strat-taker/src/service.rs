use std::sync::Arc;

use arkin_core::prelude::*;
use async_trait::async_trait;
use time::UtcDateTime;
use tracing::{info, warn};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(TypedBuilder)]
pub struct TakerExecutionStrategy {
    #[builder(default = ExecutionStrategyType::Taker)]
    strategy_type: ExecutionStrategyType,
    exec_order_book: Arc<ExecutionOrderBook>,
    venue_order_book: Arc<VenueOrderBook>,
}

impl TakerExecutionStrategy {
    pub fn new(exec_order_book: Arc<ExecutionOrderBook>, venue_order_book: Arc<VenueOrderBook>) -> Arc<Self> {
        Self {
            strategy_type: ExecutionStrategyType::Taker,
            exec_order_book,
            venue_order_book,
        }
        .into()
    }

    async fn check_finalize_exec(&self, exec_id: Uuid, event_time: UtcDateTime) {
        self.exec_order_book.check_finalize_order(exec_id, event_time).await;
    }

    async fn new_execution_order(&self, ctx: Arc<CoreCtx>, exec_order: &ExecutionOrder) {
        info!(target: "exec_strat::taker", "received new execution order {}", exec_order.id);
        let time = ctx.now().await;

        // Validate order strategy
        if exec_order.exec_strategy_type != self.strategy_type {
            warn!(target: "exec_strat::taker", "received wrong execution order type {}", exec_order.exec_strategy_type);
            return;
        }

        // add to execution order book
        self.exec_order_book.insert(exec_order.clone()).await;
        self.exec_order_book.place_order(exec_order.id, time).await;

        // Create market order
        let venue_order = VenueOrder::builder()
            .id(Uuid::new_v4())
            .execution_order_id(Some(exec_order.id))
            .strategy(exec_order.strategy.clone())
            .instrument(exec_order.instrument.clone())
            .side(exec_order.side)
            .set_quantity(exec_order.quantity)
            .set_price(exec_order.price)
            .order_type(VenueOrderType::Market)
            .created(time)
            .updated(time)
            .build();
        info!(target: "exec_strat::taker", "created new venue order {}", venue_order.id);

        // Add to the order book
        self.venue_order_book.insert(venue_order.clone()).await;

        // Publish the new venue order
        ctx.publish(Event::NewVenueOrder(venue_order.clone().into())).await;
        info!(target: "exec_strat::taker", "published new venue order {}", venue_order.id);
    }

    async fn cancel_execution_order(&self, ctx: Arc<CoreCtx>, order_id: ExecutionOrderId) {
        info!(target: "exec_strat::taker", "received cancel for execution order {}", order_id);
        let time = ctx.now().await;

        // Update the exec order book
        self.exec_order_book.cancel_order(order_id, time).await;

        // Cancel all venue orders linked to the exec order
        let venue_orders = self.venue_order_book.list_orders_by_exec_id(order_id);
        for venue_order in venue_orders {
            ctx.publish(Event::CancelVenueOrder(venue_order.clone().into())).await;
            info!(target: "exec_strat::taker", "send cancel order for venue order {}", venue_order.id);
        }
    }

    async fn cancel_all_execution_orders(&self, ctx: Arc<CoreCtx>, _time: &UtcDateTime) {
        info!(target: "exec_strat::taker", "received cancel all execution orders");
        let time = ctx.now().await;

        // Change all exec orders to cancelling
        for exec_order in self.exec_order_book.list_orders_by_exec_strategy(self.strategy_type) {
            self.exec_order_book.cancel_order(exec_order.id, time).await;
            let venue_orders = self.venue_order_book.list_orders_by_exec_id(exec_order.id);
            for venue_order in venue_orders {
                ctx.publish(Event::CancelVenueOrder(venue_order.clone().into())).await;
                info!(target: "exec_strat::taker", "send cancel order for venue order {}", venue_order.id);
            }
        }
    }

    async fn venue_order_inflight(&self, order: &VenueOrder) {
        info!(target: "exec_strat::taker", "received status inflight for venue order {}", order.id);

        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.set_inflight(order.id, order.updated).await;
            }
        }
    }

    async fn venue_order_placed(&self, order: &VenueOrder) {
        info!(target: "exec_strat::taker", "received status placed for venue order {}", order.id);

        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.place_order(order.id, order.updated).await;
            }
        }
    }

    async fn venue_order_rejected(&self, order: &VenueOrder) {
        info!(target: "exec_strat::taker", "received status rejected for venue order {}", order.id);

        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.reject_order(order.id, order.updated).await;
                if order.is_terminal() {
                    self.check_finalize_exec(id, order.updated).await;
                }
            }
        }
    }

    async fn venue_order_fill(&self, order: &VenueOrder) {
        info!(target: "exec_strat::taker", "received fill for venue order {}", order.id);

        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book
                    .add_fill_to_order(
                        order.id,
                        order.updated,
                        order.last_fill_price,
                        order.last_fill_quantity,
                        order.last_fill_commission,
                    )
                    .await;
                self.exec_order_book
                    .add_fill_to_order(
                        id,
                        order.updated,
                        order.last_fill_price,
                        order.last_fill_quantity,
                        order.last_fill_commission,
                    )
                    .await;
            }
        }
    }

    async fn venue_order_cancelled(&self, order: &VenueOrder) {
        info!(target: "exec_strat::taker", "received status cancelled for venue order {}", order.id);
        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.cancel_order(order.id, order.updated).await;
                if order.is_terminal() {
                    self.check_finalize_exec(id, order.updated).await;
                }
            }
        }
    }

    async fn venue_order_expired(&self, order: &VenueOrder) {
        info!(target: "exec_strat::taker", "received status expired for venue order {}", order.id);
        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.expire_order(order.id, order.updated).await;
                if order.is_terminal() {
                    self.check_finalize_exec(id, order.updated).await;
                }
            }
        }
    }
}

#[async_trait]
impl Runnable for TakerExecutionStrategy {
    fn event_filter(&self, _instance_type: InstanceType) -> EventFilter {
        EventFilter::Events(vec![
            EventType::NewTakerExecutionOrder,
            EventType::CancelTakerExecutionOrder,
            EventType::CancelAllTakerExecutionOrders,
            EventType::VenueOrderInflight,
            EventType::VenueOrderPlaced,
            EventType::VenueOrderRejected,
            EventType::VenueOrderFill,
            EventType::VenueOrderCancelled,
            EventType::VenueOrderExpired,
        ])
    }
    async fn handle_event(&self, ctx: Arc<CoreCtx>, event: Event) {
        match &event {
            Event::NewTakerExecutionOrder(eo) => self.new_execution_order(ctx.clone(), eo).await,
            Event::CancelTakerExecutionOrder(eo) => self.cancel_execution_order(ctx.clone(), eo.id).await,
            Event::CancelAllTakerExecutionOrders(t) => self.cancel_all_execution_orders(ctx.clone(), t).await,
            Event::VenueOrderInflight(vo) => self.venue_order_inflight(vo).await,
            Event::VenueOrderPlaced(vo) => self.venue_order_placed(vo).await,
            Event::VenueOrderRejected(vo) => self.venue_order_rejected(vo).await,
            Event::VenueOrderFill(vo) => self.venue_order_fill(vo).await,
            Event::VenueOrderCancelled(vo) => self.venue_order_cancelled(vo).await,
            Event::VenueOrderExpired(vo) => self.venue_order_expired(vo).await,
            e => warn!(target: "exec_strat::taker", "received unused event {}", e),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use arkin_core::test_utils::{MockPublisher, MockTime};
//     use rust_decimal::prelude::*;
//     use uuid::Uuid;

//     #[tokio::test]
//     #[test_log::test]
//     async fn test_new_execution_order() {
//         // Setup: Initialize OrderManager with mock dependencies
//         let time = MockTime::new();
//         let publisher = MockPublisher::new();
//         let execution_order_book = ExecutionOrderBook::new(publisher.clone(), true);
//         let venue_order_book = VenueOrderBook::new(publisher.clone(), true);
//         let exec_strategy = TakerExecutionStrategy::builder()
//             .exec_order_book(execution_order_book.to_owned())
//             .venue_order_book(venue_order_book.to_owned())
//             .build();

//         // Create a new execution order
//         let order_1 = ExecutionOrder::builder()
//             .id(Uuid::new_v4())
//             .strategy(Some(test_strategy_1()))
//             .instrument(test_inst_binance_btc_usdt_perp())
//             .exec_strategy_type(ExecutionStrategyType::Taker)
//             .side(MarketSide::Buy)
//             .set_price(dec!(0))
//             .set_quantity(dec!(1))
//             .status(ExecutionOrderStatus::New)
//             .created(time.now().await)
//             .updated(time.now().await)
//             .build();

//         // Execute: Handle the NewExecutionOrder event
//         exec_strategy
//             .handle_event(Event::NewTakerExecutionOrder(order_1.clone().into()))
//             .await;

//         // Verify: Check that the order is in the execution order book with status New
//         let retrieved_order = exec_strategy
//             .exec_order_book
//             .get(order_1.id)
//             .expect("Order should exist in the book");
//         assert_eq!(
//             retrieved_order.status,
//             ExecutionOrderStatus::Placed,
//             "Order status should be New"
//         );
//         assert_eq!(1, execution_order_book.len(), "order book should have 1 order");

//         // Create a new execution order
//         let order_2 = ExecutionOrder::builder()
//             .id(Uuid::new_v4())
//             .strategy(Some(test_strategy_1()))
//             .instrument(test_inst_binance_btc_usdt_perp())
//             .exec_strategy_type(ExecutionStrategyType::Taker)
//             .side(MarketSide::Sell)
//             .set_price(dec!(0))
//             .set_quantity(dec!(1))
//             .status(ExecutionOrderStatus::New)
//             .created(time.now().await)
//             .updated(time.now().await)
//             .build();

//         // Execute: Handle the NewExecutionOrder event
//         exec_strategy
//             .handle_event(Event::NewTakerExecutionOrder(order_2.clone().into()))
//             .await;

//         // Verify: Check that the order is in the execution order book with status New
//         let retrieved_order = exec_strategy.exec_order_book.get(order_2.id).unwrap();
//         assert_eq!(retrieved_order.status, ExecutionOrderStatus::Placed);
//         assert_eq!(2, execution_order_book.len(), "order book should have 2 order");

//         // Get venue order and fill it
//         let venue_orders = venue_order_book.list_orders();
//         for venue_order in venue_orders {
//             // Simulate Placed
//             let mut venue_placed = venue_order.clone();
//             venue_placed.place(time.now().await);
//             exec_strategy.handle_event(Event::VenueOrderPlaced(venue_placed.into())).await;

//             // Now fill
//             let mut venue_fill = venue_order.clone();
//             venue_fill.last_fill_price = dec!(100);
//             venue_fill.last_fill_quantity = dec!(1);
//             venue_fill.last_fill_commission = dec!(0.05);
//             venue_fill.filled_quantity = dec!(1);
//             venue_fill.add_fill(time.now().await, dec!(100), dec!(1), dec!(0.05));
//             exec_strategy.handle_event(Event::VenueOrderFill(venue_fill.into())).await;
//         }

//         assert_eq!(0, execution_order_book.len(), "order book should have o orders (autoclean)");
//         assert_eq!(2, publisher.get_events().await.len(), "expect 2 event (new venue order)")
//     }

//     #[tokio::test]
//     #[test_log::test]
//     async fn test_taker_flow_full_fill() {
//         // Setup: Initialize with mock dependencies
//         let time = MockTime::new();
//         let publisher = MockPublisher::new();
//         let execution_order_book = ExecutionOrderBook::new(publisher.clone(), true);
//         let venue_order_book = VenueOrderBook::new(publisher.clone(), true);
//         let exec_strategy = TakerExecutionStrategy::builder()
//             .identifier("test".to_string())
//             .time(time.clone())
//             .publisher(publisher.clone())
//             .exec_order_book(execution_order_book.clone())
//             .venue_order_book(venue_order_book.clone())
//             .build();

//         // Create exec order (Buy side)
//         let exec_order_id = Uuid::new_v4();
//         let exec_order = ExecutionOrder::builder()
//             .id(exec_order_id)
//             .strategy(Some(test_strategy_1()))
//             .instrument(test_inst_binance_btc_usdt_perp())
//             .exec_strategy_type(ExecutionStrategyType::Taker)
//             .side(MarketSide::Buy)
//             .set_price(dec!(0))
//             .set_quantity(dec!(1))
//             .status(ExecutionOrderStatus::New)
//             .created(time.now().await)
//             .updated(time.now().await)
//             .build();

//         // Handle new exec order -> set Active, place market venue immediately
//         exec_strategy
//             .handle_event(Event::NewTakerExecutionOrder(exec_order.clone().into()))
//             .await;
//         let retrieved_exec = execution_order_book.get(exec_order_id).unwrap();
//         assert_eq!(retrieved_exec.status, ExecutionOrderStatus::Placed);
//         assert_eq!(execution_order_book.len(), 1);

//         let venue_orders = venue_order_book.list_orders_by_exec_id(exec_order_id);
//         assert_eq!(venue_orders.len(), 1);
//         let venue1 = &venue_orders[0];
//         assert_eq!(venue1.order_type, VenueOrderType::Market);
//         assert_eq!(venue1.quantity, dec!(1));
//         assert_eq!(venue1.side, MarketSide::Buy);
//         assert_eq!(venue1.price, dec!(0)); // Market, no price

//         // Simulate Inflight
//         let mut venue_inflight = venue1.clone();
//         venue_inflight.set_inflight(time.now().await);
//         exec_strategy
//             .handle_event(Event::VenueOrderInflight(venue_inflight.into()))
//             .await;

//         // Simulate Placed (for market, might skip, but test anyway)
//         let mut venue_placed = venue1.clone();
//         venue_placed.place(time.now().await);
//         exec_strategy.handle_event(Event::VenueOrderPlaced(venue_placed.into())).await;

//         // Simulate full Fill
//         let mut venue_fill = venue1.clone();
//         venue_fill.last_fill_price = dec!(49500);
//         venue_fill.last_fill_quantity = dec!(1);
//         venue_fill.last_fill_commission = dec!(0.1);
//         venue_fill.filled_quantity = dec!(1);
//         venue_fill.add_fill(time.now().await, dec!(49500), dec!(1), dec!(0.1));
//         exec_strategy.handle_event(Event::VenueOrderFill(venue_fill.into())).await;

//         // Assert exec updated to Completed
//         let final_exec = execution_order_book.get(exec_order_id).unwrap();
//         assert_eq!(final_exec.filled_quantity, dec!(1));
//         assert_eq!(final_exec.status, ExecutionOrderStatus::Filled);
//     }

//     #[tokio::test]
//     #[test_log::test]
//     async fn test_taker_flow_cancel_no_fill() {
//         // Setup: Same as above
//         let time = MockTime::new();
//         let publisher = MockPublisher::new();
//         let execution_order_book = ExecutionOrderBook::new(publisher.clone(), true);
//         let venue_order_book = VenueOrderBook::new(publisher.clone(), true);
//         let exec_strategy = TakerExecutionStrategy::builder()
//             .identifier("test".to_string())
//             .time(time.clone())
//             .publisher(publisher.clone())
//             .exec_order_book(execution_order_book.clone())
//             .venue_order_book(venue_order_book.clone())
//             .build();

//         // Create and handle new exec -> Active, market venue
//         let exec_order_id = Uuid::new_v4();
//         let exec_order = ExecutionOrder::builder()
//             .id(exec_order_id)
//             .strategy(Some(test_strategy_1()))
//             .instrument(test_inst_binance_btc_usdt_perp())
//             .exec_strategy_type(ExecutionStrategyType::Taker)
//             .side(MarketSide::Sell)
//             .set_price(dec!(0))
//             .set_quantity(dec!(2))
//             .status(ExecutionOrderStatus::New)
//             .created(time.now().await)
//             .updated(time.now().await)
//             .build();

//         exec_strategy
//             .handle_event(Event::NewTakerExecutionOrder(exec_order.clone().into()))
//             .await;
//         assert_eq!(
//             execution_order_book.get(exec_order_id).unwrap().status,
//             ExecutionOrderStatus::Placed
//         );

//         let venue_orders = venue_order_book.list_orders_by_exec_id(exec_order_id);
//         assert_eq!(venue_orders.len(), 1);
//         let venue1 = &venue_orders[0];
//         assert_eq!(venue1.order_type, VenueOrderType::Market);
//         assert_eq!(venue1.side, MarketSide::Sell);

//         // Simulate Placed
//         let mut venue_placed = venue1.clone();
//         venue_placed.place(time.now().await);
//         exec_strategy
//             .handle_event(Event::VenueOrderPlaced(venue_placed.clone().into()))
//             .await;

//         // Cancel exec -> set Cancelling, publish cancel venue
//         exec_strategy
//             .handle_event(Event::CancelTakerExecutionOrder(exec_order.clone().into()))
//             .await;
//         let retrieved_exec_cancelling = execution_order_book.get(exec_order_id).unwrap();
//         assert_eq!(retrieved_exec_cancelling.status, ExecutionOrderStatus::Cancelling);

//         // Simulate Cancelled venue -> finalize to Cancelled
//         let mut venue_cancelled = venue_placed.clone();
//         venue_cancelled.cancel(time.now().await);
//         venue_cancelled.finalize_terminate(time.now().await);
//         exec_strategy
//             .handle_event(Event::VenueOrderCancelled(venue_cancelled.into()))
//             .await;

//         let final_exec = execution_order_book.get(exec_order_id).unwrap();
//         assert_eq!(final_exec.status, ExecutionOrderStatus::Cancelled);
//         assert_eq!(final_exec.filled_quantity, dec!(0));
//     }

//     #[tokio::test]
//     #[test_log::test]
//     async fn test_taker_flow_partial_fill_then_cancel() {
//         // Setup: Same
//         let time = MockTime::new();
//         let publisher = MockPublisher::new();
//         let execution_order_book = ExecutionOrderBook::new(publisher.clone(), true);
//         let venue_order_book = VenueOrderBook::new(publisher.clone(), true);
//         let exec_strategy = TakerExecutionStrategy::builder()
//             .identifier("test".to_string())
//             .time(time.clone())
//             .publisher(publisher.clone())
//             .exec_order_book(execution_order_book.clone())
//             .venue_order_book(venue_order_book.clone())
//             .build();

//         // New exec -> Active, market venue
//         let exec_order_id = Uuid::new_v4();
//         let exec_order = ExecutionOrder::builder()
//             .id(exec_order_id)
//             .strategy(Some(test_strategy_1()))
//             .instrument(test_inst_binance_btc_usdt_perp())
//             .exec_strategy_type(ExecutionStrategyType::Taker)
//             .side(MarketSide::Buy)
//             .set_price(dec!(0))
//             .set_quantity(dec!(2))
//             .status(ExecutionOrderStatus::New)
//             .created(time.now().await)
//             .updated(time.now().await)
//             .build();

//         exec_strategy
//             .handle_event(Event::NewTakerExecutionOrder(exec_order.clone().into()))
//             .await;

//         let venue_orders = venue_order_book.list_orders_by_exec_id(exec_order_id);
//         let venue1 = &venue_orders[0];

//         // Simulate Placed
//         let mut venue_placed = venue1.clone();
//         venue_placed.place(time.now().await);
//         exec_strategy
//             .handle_event(Event::VenueOrderPlaced(venue_placed.clone().into()))
//             .await;

//         // Simulate partial Fill
//         let mut venue_partial = venue_placed.clone();
//         venue_partial.add_fill(time.now().await, dec!(49500), dec!(1), dec!(0.05));
//         exec_strategy
//             .handle_event(Event::VenueOrderFill(venue_partial.clone().into()))
//             .await;

//         let exec_after_partial = execution_order_book.get(exec_order_id).unwrap();
//         assert_eq!(exec_after_partial.filled_quantity, dec!(1));
//         assert_eq!(exec_after_partial.status, ExecutionOrderStatus::Placed); // Still active

//         // Cancel -> Cancelling, cancel venue
//         exec_strategy
//             .handle_event(Event::CancelTakerExecutionOrder(exec_order.clone().into()))
//             .await;
//         assert_eq!(
//             execution_order_book.get(exec_order_id).unwrap().status,
//             ExecutionOrderStatus::Cancelling
//         );

//         // Simulate Cancelled venue (with partial fill) -> finalize (note: current logic sets Cancelled, not Partial; flaw?)
//         let mut venue_cancelled = venue_partial.clone();
//         venue_cancelled.filled_quantity = dec!(1); // Preserve partial
//         venue_cancelled.cancel(time.now().await);
//         venue_cancelled.finalize_terminate(time.now().await);
//         exec_strategy
//             .handle_event(Event::VenueOrderCancelled(venue_cancelled.into()))
//             .await;

//         let final_exec = execution_order_book.get(exec_order_id).unwrap();
//         assert_eq!(final_exec.status, ExecutionOrderStatus::PartiallyFilledCancelled); // Assume updated logic
//         assert_eq!(final_exec.filled_quantity, dec!(1));
//     }
// }
