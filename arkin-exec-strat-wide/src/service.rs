use std::{collections::HashMap, sync::Arc, time::Duration};

use arkin_core::prelude::*;
use async_trait::async_trait;
use rust_decimal::prelude::*;
use time::UtcDateTime;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};
use typed_builder::TypedBuilder;
use uuid::Uuid;

#[derive(TypedBuilder)]
pub struct WideQuoterExecutionStrategy {
    identifier: String,
    #[builder(default = ExecutionStrategyType::WideQuoter)]
    strategy_type: ExecutionStrategyType,
    time: Arc<dyn SystemTime>,
    publisher: Arc<dyn Publisher>,
    exec_order_book: Arc<ExecutionOrderBook>,
    venue_order_book: Arc<VenueOrderBook>,
    #[builder(default)]
    last_quoted_mids: Mutex<HashMap<Uuid, Decimal>>,
    pct_from_mid: Decimal,
    requote_pct_change: Decimal,
}

impl WideQuoterExecutionStrategy {
    pub fn new(
        identifier: &str,
        time: Arc<dyn SystemTime>,
        publisher: Arc<dyn Publisher>,
        exec_order_book: Arc<ExecutionOrderBook>,
        venue_order_book: Arc<VenueOrderBook>,
        pct_from_mid: Decimal,
        requote_pct_change: Decimal,
    ) -> Arc<Self> {
        Self {
            identifier: identifier.to_owned(),
            strategy_type: ExecutionStrategyType::WideQuoter,
            time,
            publisher,
            exec_order_book,
            venue_order_book,
            last_quoted_mids: Mutex::new(HashMap::new()),
            pct_from_mid,
            requote_pct_change,
        }
        .into()
    }

    #[instrument(skip_all)]
    async fn check_finalize_exec(&self, exec_id: Uuid) {
        let now = self.time.now().await;
        self.exec_order_book.finalize_terminate_order(exec_id, now).await;
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn new_execution_order(&self, exec_order: &ExecutionOrder) {
        info!(target: "exec_strat::wide", "received new execution order {}", exec_order.id);

        // Validate order strategy
        if exec_order.exec_strategy_type != self.strategy_type {
            warn!(target: "exec_strat::wide", "received wrong execution order type {}", exec_order.exec_strategy_type);
            return;
        }

        // add to execution order book
        self.exec_order_book.insert(exec_order.clone()).await;
        self.exec_order_book.place_order(exec_order.id, self.time.now().await).await;
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn cancel_execution_order(&self, order_id: ExecutionOrderId) {
        info!(target: "exec_strat::wide", "received cancel for execution order {}", order_id);

        // Update the exec order book
        self.exec_order_book.cancel_order(order_id, self.time.now().await).await;

        // Cancel all venue orders linked to the exec order
        let venue_orders = self.venue_order_book.list_orders_by_exec_id(order_id);
        for venue_order in venue_orders {
            self.publisher
                .publish(Event::CancelVenueOrder(venue_order.clone().into()))
                .await;
            info!(target: "exec_strat::wide", "send cancel order for venue order {}", venue_order.id);
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn cancel_all_execution_orders(&self, _time: &UtcDateTime) {
        info!(target: "exec_strat::wide", "received cancel all execution orders");

        // Change all exec orders to cancelling
        let now = self.time.now().await;
        for exec_order in self.exec_order_book.list_orders_by_exec_strategy(self.strategy_type) {
            self.exec_order_book.cancel_order(exec_order.id, now).await;
            let venue_orders = self.venue_order_book.list_orders_by_exec_id(exec_order.id);
            for venue_order in venue_orders {
                self.venue_order_book.cancel_order(venue_order.id, now).await;
                self.publisher
                    .publish(Event::CancelVenueOrder(venue_order.clone().into()))
                    .await;
                info!(target: "exec_strat::wide", "send cancel order for venue order {}", venue_order.id);
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_inflight(&self, order: &VenueOrder) {
        info!(target: "exec_strat::wide", "received status inflight for venue order {}", order.id);

        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.set_inflight(order.id, order.updated).await;
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_placed(&self, order: &VenueOrder) {
        info!(target: "exec_strat::wide", "received status placed for venue order {}", order.id);

        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.place_order(order.id, order.updated).await;
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_rejected(&self, order: &VenueOrder) {
        info!(target: "exec_strat::wide", "received status rejected for venue order {}", order.id);

        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.reject_order(order.id, order.updated).await;
            }
            self.check_finalize_exec(id).await;
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_fill(&self, order: &VenueOrder) {
        info!(target: "exec_strat::wide", "received fill for venue order {}", order.id);

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
            self.check_finalize_exec(id).await;
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_cancelled(&self, order: &VenueOrder) {
        info!(target: "exec_strat::wide", "received status cancelled for venue order {}", order.id);
        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.finalize_terminate_order(order.id, order.updated).await;
            }
            self.check_finalize_exec(id).await;
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn venue_order_expired(&self, order: &VenueOrder) {
        info!(target: "exec_strat::wide", "received status expired for venue order {}", order.id);
        // Check if the order contains exec id and if we are the right strategy
        if let Some(id) = order.execution_order_id {
            let exec_ids = self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type);
            if order.execution_order_id.is_some() && exec_ids.contains(&id) {
                self.venue_order_book.expire_order(order.id, order.updated).await;
            }
            self.check_finalize_exec(id).await;
        }
    }

    async fn tick_update(&self, tick: &Tick) {
        debug!(target: "exec_strat::wide", "received tick update for {}", tick.instrument);

        let mid = (tick.bid_price + tick.ask_price) / dec!(2);
        let exec_orders = self
            .exec_order_book
            .list_active_orders_by_instrument_and_strategy(&tick.instrument, self.strategy_type);

        debug!(target: "exec_strat::wide", "found {} execution orders", exec_orders.len());
        for exec_order in exec_orders {
            let venue_orders = self.venue_order_book.list_orders_by_exec_id(exec_order.id);
            let has_active_venue = !venue_orders.is_empty() && venue_orders.iter().any(|v| v.is_active());

            let desired_price = if exec_order.side == MarketSide::Buy {
                mid * (Decimal::ONE - self.pct_from_mid)
            } else {
                mid * (Decimal::ONE + self.pct_from_mid)
            };

            let mut last_mids = self.last_quoted_mids.lock().await;
            let last_mid = last_mids.get(&exec_order.id).cloned().unwrap_or(Decimal::ZERO);

            let pct_change = if last_mid != Decimal::ZERO {
                ((mid - last_mid) / last_mid).abs()
            } else {
                Decimal::ONE // Force initial placement
            };
            debug!(target: "exec_strat::wide", "Checking for exec order {}, with {} venue_orders", exec_order, venue_orders.len());

            if !has_active_venue || pct_change > self.requote_pct_change {
                // Cancel existing if any
                for venue_order in venue_orders {
                    if venue_order.is_active() {
                        self.venue_order_book.cancel_order(venue_order.id, self.time.now().await).await;
                        self.publisher
                            .publish(Event::CancelVenueOrder(venue_order.clone().into()))
                            .await;
                        info!(target: "exec_strat::wide", "cancelling venue order {} for requote", venue_order.id);
                    }
                }

                // Create new limit order
                let venue_order = VenueOrder::builder()
                    .id(Uuid::new_v4())
                    .execution_order_id(Some(exec_order.id))
                    .strategy(exec_order.strategy.clone())
                    .instrument(exec_order.instrument.clone())
                    .side(exec_order.side)
                    .set_price(desired_price)
                    .set_quantity(exec_order.remaining_quantity()) // Handle partial fills
                    .order_type(VenueOrderType::Limit)
                    .created(self.time.now().await)
                    .updated(self.time.now().await)
                    .build();
                self.venue_order_book.insert(venue_order.clone()).await;
                self.publisher.publish(Event::NewVenueOrder(venue_order.clone().into())).await;
                info!(target: "exec_strat::wide", "placed new venue order {} at {}", venue_order.id, desired_price);

                // Update last_mid
                last_mids.insert(exec_order.id, mid);
            }
        }
    }
}
#[async_trait]
impl Runnable for WideQuoterExecutionStrategy {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            Event::NewWideQuoterExecutionOrder(eo) => self.new_execution_order(eo).await,
            Event::CancelWideQuoterExecutionOrder(eo) => self.cancel_execution_order(eo.id).await,
            Event::CancelAllWideQuoterExecutionOrders(t) => self.cancel_all_execution_orders(t).await,
            Event::VenueOrderInflight(vo) => self.venue_order_inflight(vo).await,
            Event::VenueOrderPlaced(vo) => self.venue_order_placed(vo).await,
            Event::VenueOrderRejected(vo) => self.venue_order_rejected(vo).await,
            Event::VenueOrderFill(vo) => self.venue_order_fill(vo).await,
            Event::VenueOrderCancelled(vo) => self.venue_order_cancelled(vo).await,
            Event::VenueOrderExpired(vo) => self.venue_order_expired(vo).await,
            Event::TickUpdate(t) => self.tick_update(t).await,
            e => warn!(target: "exec_strat::wide", "received unused event {}", e),
        }
    }

    #[instrument(skip_all, fields(service = %self.identifier))]
    async fn teardown(&self, _ctx: Arc<ServiceCtx>) {
        self.cancel_all_execution_orders(&self.time.now().await).await;

        while !self.exec_order_book.list_ids_by_exec_strategy(self.strategy_type).is_empty() {
            info!(target: "exec_strat::wide", "waiting for orders to cancel");
            let exec_orders = self.exec_order_book.list_orders_by_exec_strategy(self.strategy_type);

            info!(target: "exec_strat::wide", "--- EXEC ORDERS ---");
            for order in exec_orders {
                info!(target: "exec_strat::wide", " - {}", order);
            }

            info!(target: "exec_strat::wide", "--- VENUE ORDERS ---");
            let venue_orders = self.venue_order_book.list_orders();
            for order in venue_orders {
                info!(target: "exec_strat::wide", " - {}", order);
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkin_core::test_utils::{MockPublisher, MockTime};
    use uuid::Uuid;

    #[tokio::test]
    #[test_log::test]
    async fn test_wide_quoter_flow_buy_side() {
        // Setup: Initialize with mock dependencies
        let time = MockTime::new();
        let publisher = MockPublisher::new();
        let execution_order_book = ExecutionOrderBook::new(publisher.clone(), false);
        let venue_order_book = VenueOrderBook::new(publisher.clone(), false);
        let exec_strategy = WideQuoterExecutionStrategy::builder()
            .identifier("test".to_string())
            .time(time.clone())
            .publisher(publisher.clone())
            .exec_order_book(execution_order_book.clone())
            .venue_order_book(venue_order_book.clone())
            .pct_from_mid(dec!(0.01))
            .requote_pct_change(dec!(0.002))
            .build();

        // Create exec order (Buy side)
        let exec_order_id = Uuid::new_v4();
        let exec_order = ExecutionOrder::builder()
            .id(exec_order_id)
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .exec_strategy_type(ExecutionStrategyType::WideQuoter)
            .side(MarketSide::Buy)
            .set_price(dec!(0))
            .set_quantity(dec!(1))
            .status(ExecutionOrderStatus::New)
            .created(time.now().await)
            .updated(time.now().await)
            .build();

        // Handle new exec order -> should set Active, no venue yet
        exec_strategy
            .handle_event(Event::NewWideQuoterExecutionOrder(exec_order.clone().into()))
            .await;
        let retrieved_exec = execution_order_book.get(exec_order_id).unwrap();
        assert_eq!(retrieved_exec.status, ExecutionOrderStatus::Placed);
        assert_eq!(execution_order_book.len(), 1);
        assert_eq!(venue_order_book.len(), 0); // No venue until tick

        // Tick 1: mid = (49000 + 50000)/2 = 49500, desired buy = 49500 * 0.99 = 49005
        // Should place initial Limit venue order
        let tick1 = Tick::builder()
            .event_time(time.now().await)
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(0u64)
            .ask_price(dec!(50000.0))
            .ask_quantity(dec!(0.3))
            .bid_price(dec!(49000.0))
            .bid_quantity(dec!(0.7))
            .build();
        exec_strategy.handle_event(Event::TickUpdate(tick1.clone().into())).await;

        let venue_orders = venue_order_book.list_orders_by_exec_id(exec_order_id);
        assert_eq!(venue_orders.len(), 1);
        let venue1 = &venue_orders[0];
        assert_eq!(venue1.order_type, VenueOrderType::Limit);
        assert_eq!(venue1.price, dec!(49005)); // Verify calc
        assert_eq!(venue1.quantity, dec!(1));
        assert_eq!(venue1.side, MarketSide::Buy);

        // Simulate Placed (update book, but no logic change)
        let mut venue1_placed = venue1.clone();
        venue1_placed.place(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderPlaced(venue1_placed.clone().into()))
            .await;

        // Tick 2: small change mid=49549, pct≈0.00099<0.002 -> no requote
        let tick2 = Tick::builder()
            .event_time(time.now().await)
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(1u64)
            .ask_price(dec!(50049.0))
            .ask_quantity(dec!(0.3))
            .bid_price(dec!(49049.0))
            .bid_quantity(dec!(0.7))
            .build();
        exec_strategy.handle_event(Event::TickUpdate(tick2.clone().into())).await;
        assert_eq!(venue_order_book.list_orders_by_exec_id(exec_order_id).len(), 1); // No requote
        assert_eq!(venue_order_book.get(venue1_placed.id).unwrap().price, dec!(49005)); // Unchanged

        // Tick 3: large move, mid=52500, pct>0.002 -> requote at 52500*0.99=51975
        let tick3 = Tick::builder()
            .event_time(time.now().await)
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(2u64)
            .ask_price(dec!(53000.0))
            .ask_quantity(dec!(0.3))
            .bid_price(dec!(52000.0))
            .bid_quantity(dec!(0.7))
            .build();
        exec_strategy.handle_event(Event::TickUpdate(tick3.clone().into())).await;

        let venue_orders_after = venue_order_book.list_orders_by_exec_id(exec_order_id);
        assert_eq!(venue_orders_after.len(), 2); // Old + new
        let new_venue = venue_orders_after.iter().find(|v| v.price == dec!(51975)).unwrap();
        assert_eq!(new_venue.order_type, VenueOrderType::Limit);

        // Simulate Placed for new venue
        let mut new_venue_placed = new_venue.clone();
        new_venue_placed.place(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderPlaced(new_venue_placed.clone().into()))
            .await;

        // Simulate cancel of old venue
        let mut old_venue_cancelled = venue1_placed.clone();
        old_venue_cancelled.cancel(time.now().await);
        old_venue_cancelled.finalize_terminate(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderCancelled(old_venue_cancelled.into()))
            .await;

        // Cancel exec order -> should cancel active venue, set exec Cancelling
        exec_strategy
            .handle_event(Event::CancelWideQuoterExecutionOrder(exec_order.clone().into()))
            .await;
        let retrieved_exec_cancelling = execution_order_book.get(exec_order_id).unwrap();
        assert_eq!(retrieved_exec_cancelling.status, ExecutionOrderStatus::Cancelling);

        // Simulate cancelled for remaining venue -> should set exec Cancelled
        let mut new_venue_cancelled = new_venue_placed.clone();
        new_venue_cancelled.cancel(time.now().await);
        new_venue_cancelled.finalize_terminate(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderCancelled(new_venue_cancelled.into()))
            .await;

        let final_exec = execution_order_book.get(exec_order_id).unwrap();
        assert_eq!(final_exec.status, ExecutionOrderStatus::Cancelled);
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_wide_quoter_flow_sell_side() {
        // Setup: Initialize with mock dependencies
        let time = MockTime::new();
        let publisher = MockPublisher::new();
        let execution_order_book = ExecutionOrderBook::new(publisher.clone(), false);
        let venue_order_book = VenueOrderBook::new(publisher.clone(), false);
        let exec_strategy = WideQuoterExecutionStrategy::builder()
            .identifier("test".to_string())
            .time(time.clone())
            .publisher(publisher.clone())
            .exec_order_book(execution_order_book.clone())
            .venue_order_book(venue_order_book.clone())
            .pct_from_mid(dec!(0.01))
            .requote_pct_change(dec!(0.002))
            .build();

        // Create exec order (Sell side)
        let exec_order_id = Uuid::new_v4();
        let exec_order = ExecutionOrder::builder()
            .id(exec_order_id)
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .exec_strategy_type(ExecutionStrategyType::WideQuoter)
            .side(MarketSide::Sell)
            .set_price(dec!(0))
            .set_quantity(dec!(1))
            .status(ExecutionOrderStatus::New)
            .created(time.now().await)
            .updated(time.now().await)
            .build();

        // Handle new exec order -> should set Active, no venue yet
        exec_strategy
            .handle_event(Event::NewWideQuoterExecutionOrder(exec_order.clone().into()))
            .await;
        let retrieved_exec = execution_order_book.get(exec_order_id).unwrap();
        assert_eq!(retrieved_exec.status, ExecutionOrderStatus::Placed);
        assert_eq!(execution_order_book.len(), 1);
        assert_eq!(venue_order_book.len(), 0); // No venue until tick

        // Tick 1: mid = (49000 + 50000)/2 = 49500, desired sell = 49500 * 1.01 = 49995
        // Should place initial Limit venue order
        let tick1 = Tick::builder()
            .event_time(time.now().await)
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(0u64)
            .ask_price(dec!(50000.0))
            .ask_quantity(dec!(0.3))
            .bid_price(dec!(49000.0))
            .bid_quantity(dec!(0.7))
            .build();
        exec_strategy.handle_event(Event::TickUpdate(tick1.clone().into())).await;

        let venue_orders = venue_order_book.list_orders_by_exec_id(exec_order_id);
        assert_eq!(venue_orders.len(), 1);
        let venue1 = &venue_orders[0];
        assert_eq!(venue1.order_type, VenueOrderType::Limit);
        assert_eq!(venue1.price, dec!(49995)); // Verify calc
        assert_eq!(venue1.quantity, dec!(1));
        assert_eq!(venue1.side, MarketSide::Sell);

        // Simulate Placed (update book, but no logic change)
        let mut venue1_placed = venue1.clone();
        venue1_placed.place(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderPlaced(venue1_placed.clone().into()))
            .await;

        // Tick 2: small change mid=49549, pct≈0.00099<0.002 -> no requote
        let tick2 = Tick::builder()
            .event_time(time.now().await)
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(1u64)
            .ask_price(dec!(50049.0))
            .ask_quantity(dec!(0.3))
            .bid_price(dec!(49049.0))
            .bid_quantity(dec!(0.7))
            .build();
        exec_strategy.handle_event(Event::TickUpdate(tick2.clone().into())).await;
        assert_eq!(venue_order_book.list_orders_by_exec_id(exec_order_id).len(), 1); // No requote
        assert_eq!(venue_order_book.get(venue1.id).unwrap().price, dec!(49995)); // Unchanged

        // Tick 3: large move, mid=52500, pct>0.002 -> requote at 52500*1.01=53025
        let tick3 = Tick::builder()
            .event_time(time.now().await)
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(2u64)
            .ask_price(dec!(53000.0))
            .ask_quantity(dec!(0.3))
            .bid_price(dec!(52000.0))
            .bid_quantity(dec!(0.7))
            .build();
        exec_strategy.handle_event(Event::TickUpdate(tick3.clone().into())).await;

        let venue_orders_after = venue_order_book.list_orders_by_exec_id(exec_order_id);
        assert_eq!(venue_orders_after.len(), 2); // Old + new
        let new_venue = venue_orders_after.iter().find(|v| v.price == dec!(53025)).unwrap();
        assert_eq!(new_venue.order_type, VenueOrderType::Limit);

        // Simulate Placed for new venue
        let mut new_venue_placed = new_venue.clone();
        new_venue_placed.place(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderPlaced(new_venue_placed.clone().into()))
            .await;

        // Simulate cancel of old venue
        let mut old_venue_cancelled = venue1_placed.clone();
        old_venue_cancelled.cancel(time.now().await);
        old_venue_cancelled.finalize_terminate(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderCancelled(old_venue_cancelled.into()))
            .await;

        // Cancel exec order -> should cancel active venue, set exec Cancelling
        exec_strategy
            .handle_event(Event::CancelWideQuoterExecutionOrder(exec_order.clone().into()))
            .await;
        let retrieved_exec_cancelling = execution_order_book.get(exec_order_id).unwrap();
        assert_eq!(retrieved_exec_cancelling.status, ExecutionOrderStatus::Cancelling);

        // Simulate cancelled for remaining venue -> should set exec Cancelled
        let mut new_venue_cancelled = new_venue_placed.clone();
        new_venue_cancelled.cancel(time.now().await);
        new_venue_cancelled.finalize_terminate(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderCancelled(new_venue_cancelled.into()))
            .await;

        let final_exec = execution_order_book.get(exec_order_id).unwrap();
        assert_eq!(final_exec.status, ExecutionOrderStatus::Cancelled);
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_wide_quoter_flow_two_sided() {
        // Setup: Initialize with mock dependencies
        let time = MockTime::new();
        let publisher = MockPublisher::new();
        let execution_order_book = ExecutionOrderBook::new(publisher.clone(), false);
        let venue_order_book = VenueOrderBook::new(publisher.clone(), false);
        let exec_strategy = WideQuoterExecutionStrategy::builder()
            .identifier("test".to_string())
            .time(time.clone())
            .publisher(publisher.clone())
            .exec_order_book(execution_order_book.clone())
            .venue_order_book(venue_order_book.clone())
            .pct_from_mid(dec!(0.01))
            .requote_pct_change(dec!(0.002))
            .build();

        // Create Buy exec order
        let buy_exec_id = Uuid::new_v4();
        let buy_exec = ExecutionOrder::builder()
            .id(buy_exec_id)
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .exec_strategy_type(ExecutionStrategyType::WideQuoter)
            .side(MarketSide::Buy)
            .set_price(dec!(0))
            .set_quantity(dec!(1))
            .status(ExecutionOrderStatus::New)
            .created(time.now().await)
            .updated(time.now().await)
            .build();

        exec_strategy
            .handle_event(Event::NewWideQuoterExecutionOrder(buy_exec.clone().into()))
            .await;
        assert_eq!(
            execution_order_book.get(buy_exec_id).unwrap().status,
            ExecutionOrderStatus::Placed
        );

        // Create Sell exec order
        let sell_exec_id = Uuid::new_v4();
        let sell_exec = ExecutionOrder::builder()
            .id(sell_exec_id)
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .exec_strategy_type(ExecutionStrategyType::WideQuoter)
            .side(MarketSide::Sell)
            .set_price(dec!(0))
            .set_quantity(dec!(1))
            .status(ExecutionOrderStatus::New)
            .created(time.now().await)
            .updated(time.now().await)
            .build();

        exec_strategy
            .handle_event(Event::NewWideQuoterExecutionOrder(sell_exec.clone().into()))
            .await;
        assert_eq!(
            execution_order_book.get(sell_exec_id).unwrap().status,
            ExecutionOrderStatus::Placed
        );
        assert_eq!(execution_order_book.len(), 2);
        assert_eq!(venue_order_book.len(), 0);

        // Tick 1: Place Buy@49005 and Sell@49995
        let tick1 = Tick::builder()
            .event_time(time.now().await)
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(0u64)
            .ask_price(dec!(50000.0))
            .ask_quantity(dec!(0.3))
            .bid_price(dec!(49000.0))
            .bid_quantity(dec!(0.7))
            .build();
        exec_strategy.handle_event(Event::TickUpdate(tick1.clone().into())).await;

        let buy_venues = venue_order_book.list_orders_by_exec_id(buy_exec_id);
        assert_eq!(buy_venues.len(), 1);
        let buy_venue1 = &buy_venues[0];
        assert_eq!(buy_venue1.price, dec!(49005));
        assert_eq!(buy_venue1.side, MarketSide::Buy);

        let sell_venues = venue_order_book.list_orders_by_exec_id(sell_exec_id);
        assert_eq!(sell_venues.len(), 1);
        let sell_venue1 = &sell_venues[0];
        assert_eq!(sell_venue1.price, dec!(49995));
        assert_eq!(sell_venue1.side, MarketSide::Sell);

        // Simulate Placed for both
        let mut buy_placed = buy_venue1.clone();
        buy_placed.place(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderPlaced(buy_placed.clone().into()))
            .await;

        let mut sell_placed = sell_venue1.clone();
        sell_placed.place(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderPlaced(sell_placed.clone().into()))
            .await;

        // Tick 2: No requote
        let tick2 = Tick::builder()
            .event_time(time.now().await)
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(1u64)
            .ask_price(dec!(50049.0))
            .ask_quantity(dec!(0.3))
            .bid_price(dec!(49049.0))
            .bid_quantity(dec!(0.7))
            .build();
        exec_strategy.handle_event(Event::TickUpdate(tick2.clone().into())).await;
        assert_eq!(venue_order_book.list_orders_by_exec_id(buy_exec_id).len(), 1);
        assert_eq!(venue_order_book.list_orders_by_exec_id(sell_exec_id).len(), 1);

        // Tick 3: Requote both
        let tick3 = Tick::builder()
            .event_time(time.now().await)
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(2u64)
            .ask_price(dec!(53000.0))
            .ask_quantity(dec!(0.3))
            .bid_price(dec!(52000.0))
            .bid_quantity(dec!(0.7))
            .build();
        exec_strategy.handle_event(Event::TickUpdate(tick3.clone().into())).await;

        let buy_venues_after = venue_order_book.list_orders_by_exec_id(buy_exec_id);
        assert_eq!(buy_venues_after.len(), 2);
        let new_buy = buy_venues_after.iter().find(|v| v.price == dec!(51975)).unwrap();

        let sell_venues_after = venue_order_book.list_orders_by_exec_id(sell_exec_id);
        assert_eq!(sell_venues_after.len(), 2);
        let new_sell = sell_venues_after.iter().find(|v| v.price == dec!(53025)).unwrap();

        // Simulate Placed for new venues
        let mut new_buy_placed = new_buy.clone();
        new_buy_placed.place(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderPlaced(new_buy_placed.clone().into()))
            .await;

        let mut new_sell_placed = new_sell.clone();
        new_sell_placed.place(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderPlaced(new_sell_placed.clone().into()))
            .await;

        // Simulate cancel old
        let mut old_buy_cancel = buy_placed.clone();
        old_buy_cancel.cancel(time.now().await);
        old_buy_cancel.finalize_terminate(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderCancelled(old_buy_cancel.into()))
            .await;

        let mut old_sell_cancel = sell_placed.clone();
        old_sell_cancel.cancel(time.now().await);
        old_sell_cancel.finalize_terminate(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderCancelled(old_sell_cancel.into()))
            .await;

        // Cancel Buy exec
        exec_strategy
            .handle_event(Event::CancelWideQuoterExecutionOrder(buy_exec.clone().into()))
            .await;
        assert_eq!(
            execution_order_book.get(buy_exec_id).unwrap().status,
            ExecutionOrderStatus::Cancelling
        );

        // Cancel Sell exec
        exec_strategy
            .handle_event(Event::CancelWideQuoterExecutionOrder(sell_exec.clone().into()))
            .await;
        assert_eq!(
            execution_order_book.get(sell_exec_id).unwrap().status,
            ExecutionOrderStatus::Cancelling
        );

        // Simulate cancel new Buy
        let mut new_buy_cancel = new_buy_placed.clone();
        new_buy_cancel.cancel(time.now().await);
        new_buy_cancel.finalize_terminate(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderCancelled(new_buy_cancel.into()))
            .await;
        assert_eq!(
            execution_order_book.get(buy_exec_id).unwrap().status,
            ExecutionOrderStatus::Cancelled
        );

        // Simulate cancel new Sell
        let mut new_sell_cancel = new_sell_placed.clone();
        new_sell_cancel.cancel(time.now().await);
        new_sell_cancel.finalize_terminate(time.now().await);
        exec_strategy
            .handle_event(Event::VenueOrderCancelled(new_sell_cancel.into()))
            .await;
        assert_eq!(
            execution_order_book.get(sell_exec_id).unwrap().status,
            ExecutionOrderStatus::Cancelled
        );
    }
}
