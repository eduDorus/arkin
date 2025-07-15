use std::{cmp::min, sync::Arc};

use async_trait::async_trait;
use dashmap::DashMap;
use rust_decimal::prelude::*;
use tracing::{debug, info, instrument, warn};

use arkin_core::prelude::*;

use crate::book::ExchangeBook;

#[derive(Clone, Default)]
struct InternalAccount {
    balances: DashMap<Arc<Asset>, Decimal>,                  // asset -> wallet balance
    positions: DashMap<Arc<Instrument>, (Decimal, Decimal)>, // instrument -> (entry_price, position)
}

impl InternalAccount {
    fn update_balance(&self, asset: &Arc<Asset>, quantity: Decimal) {
        *self.balances.entry(asset.to_owned()).or_insert(Decimal::ZERO) += quantity;
    }

    fn update_position(&self, instrument: &Arc<Instrument>, entry_price: Decimal, quantity: Decimal) {
        *self
            .positions
            .entry(instrument.to_owned())
            .or_insert((Decimal::ZERO, Decimal::ZERO)) = (entry_price, quantity);
    }
}

pub struct Executor {
    identifier: String,
    time: Arc<dyn SystemTime>,
    publisher: Arc<dyn Publisher>,
    venue: Arc<Venue>,
    orderbook: ExchangeBook,
    account: InternalAccount,
    taker_commission: Decimal,
    maker_commission: Decimal,
    leverage: Decimal,
}

impl Executor {
    pub fn new(identifier: &str, time: Arc<dyn SystemTime>, publisher: Arc<dyn Publisher>) -> Arc<Self> {
        let account = InternalAccount::default();
        account.update_balance(&test_usdt_asset(), dec!(100000));
        Self {
            identifier: identifier.to_owned(),
            time,
            publisher,
            venue: test_binance_venue(), // TODO: This will not work in prod
            orderbook: ExchangeBook::default(),
            account,
            taker_commission: dec!(0.0005),
            maker_commission: dec!(0.0002),
            leverage: dec!(10),
        }
        .into()
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn place_order(&self, order: &VenueOrder) {
        info!(target: "executor-simulation", "received new order");

        info!(target: "executor-simulation", "change order to inflight and add order {} to orderbook", order.id);
        let mut order = order.clone();
        let time = self.time.now().await;
        order.set_inflight(time);
        self.orderbook.insert(order.clone());
        self.publisher.publish(Event::VenueOrderInflight(order.clone().into())).await;

        // TODO: Check if we have enough funds
        info!(target: "executor-simulation", "change order to placed and sending placed event for order {}", order.id);
        let time = self.time.now().await;
        order.place(time);
        self.orderbook.update(order.clone());
        self.publisher.publish(Event::VenueOrderPlaced(order.into())).await;
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn cancel_order(&self, order: &VenueOrder) {
        info!(target: "executor-simulation", "received cancel order");

        if let Some((_, order)) = self.orderbook.remove(order.id) {
            info!(target: "executor-simulation", "order {} successfully cancelled", order.id);
            self.publisher.publish(Event::VenueOrderCancelled(order.into())).await;
        } else {
            warn!(target: "executor-simulation", "order {} not in order book, could not cancel", order.id);
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn cancel_all(&self) {
        info!(target: "executor-simulation", "received cancel all orders");

        let orders = self.orderbook.list_orders();
        for order in orders {
            if let Some((_, order)) = self.orderbook.remove(order.id) {
                self.publisher.publish(Event::VenueOrderCancelled(order.into())).await;
            } else {
                warn!(target: "executor-simulation", "order {} not in order book, could not cancel", order.id);
            }
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    fn check_market_order(&self, tick: &Tick, order: &VenueOrder) -> (bool, Decimal, Decimal, Decimal) {
        let (matched, price, quantity) = match order.side {
            MarketSide::Buy => (true, tick.ask_price(), min(order.remaining_quantity(), tick.ask_quantity)),
            MarketSide::Sell => (true, tick.bid_price(), min(order.remaining_quantity(), tick.bid_quantity)),
        };
        let commission = price * quantity * self.taker_commission;
        (matched, price, quantity, commission)
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    fn check_limit_order(&self, tick: &Tick, order: &VenueOrder) -> (bool, Decimal, Decimal, Decimal) {
        let (matched, price, quantity) = match order.side {
            MarketSide::Buy => (
                tick.ask_price() <= order.price,
                tick.ask_price(),
                min(order.remaining_quantity(), tick.ask_quantity),
            ),
            MarketSide::Sell => (
                tick.bid_price() >= order.price,
                tick.bid_price(),
                min(order.remaining_quantity(), tick.bid_quantity),
            ),
        };
        let commission = price * quantity * self.maker_commission;
        (matched, price, quantity, commission)
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn fill_update(&self, order: &VenueOrder, price: Decimal, quantity: Decimal, commission: Decimal) {
        info!(target: "executor-simulation", "matched order {} at {} with {} commissions {}", order.id, order.last_fill_quantity, order.last_fill_price, order.last_fill_commission);

        // Add the fill
        let mut order = order.clone();
        order.add_fill(self.time.now().await, price, quantity, commission);

        // Check if the order is fully filled
        if order.remaining_quantity().is_zero() {
            info!(target: "executor-simulation", "order {} filled with total of {}@{} commission {}", order.id, order.quantity, order.filled_price, order.commission);
            self.orderbook.remove(order.id);
        } else {
            info!(target: "executor-simulation", "order {} partially filled {} with total of {}/{}", order.id, order.last_fill_quantity, order.filled_quantity, order.quantity);
            self.orderbook.update(order.clone());
        }

        self.publisher.publish(Event::VenueOrderFill(order.to_owned().into())).await;
        info!(target: "executor-simulation", "published fill update for {}", order.id);
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn account_update(&self, order: &VenueOrder, price: Decimal, quantity: Decimal, commission: Decimal) {
        let instrument = order.instrument.clone();
        let margin_asset = order.instrument.margin_asset.clone();
        let side_sign: Decimal = order.side.into();
        let qty_delta = quantity * side_sign;

        // Get current pos/entry
        let (entry, current_qty) = self
            .account
            .positions
            .get(&instrument)
            .map_or((dec!(0), dec!(0)), |p| *p.value());

        // Calc closed amt/PNL if reducing
        let amt_closed = if current_qty.signum() != qty_delta.signum() || current_qty.abs() < quantity {
            current_qty.abs().min(quantity)
        } else {
            dec!(0)
        };
        let pnl = if amt_closed > dec!(0) {
            let pnl_sign = if current_qty > dec!(0) {
                dec!(1)
            } else {
                dec!(-1)
            };
            (price - entry) * amt_closed * pnl_sign
        } else {
            dec!(0)
        };

        // New qty/entry
        let new_qty = current_qty + qty_delta;
        let new_entry = if new_qty == dec!(0) {
            dec!(0)
        } else if amt_closed == current_qty.abs() {
            // Flip: new entry = price
            price
        } else {
            // Avg
            (entry * current_qty.abs() + price * (quantity - amt_closed)) / new_qty.abs()
        };

        // Margin delta (change to wallet: negative for post, positive for release)
        let margin_release = if amt_closed > dec!(0) {
            amt_closed * entry / self.leverage
        } else {
            dec!(0)
        };
        let margin_post = if qty_delta.abs() > amt_closed {
            (qty_delta.abs() - amt_closed) * price / self.leverage
        } else {
            dec!(0)
        };
        let margin_delta = margin_release - margin_post;

        // Update internal
        self.account.update_balance(&margin_asset, pnl + margin_delta - commission); // Net to balance
        self.account.update_position(&instrument, new_entry, new_qty);

        // Publish AccountUpdate
        let bal_change = pnl + margin_delta - commission;
        let new_bal = self.account.balances.get(&margin_asset).map_or(dec!(0), |b| *b.value());
        let bal_update = BalanceUpdate::builder()
            .event_time(self.time.now().await)
            .venue(instrument.venue.clone())
            .asset(margin_asset)
            .account_type(AccountType::Margin)
            .quantity_change(bal_change)
            .quantity(new_bal)
            .build();

        let pos_update = PositionUpdate::builder()
            .event_time(self.time.now().await)
            .instrument(instrument)
            .account_type(AccountType::Margin)
            .entry_price(new_entry)
            .quantity(new_qty)
            .realized_pnl(pnl) // Incremental
            .unrealized_pnl(dec!(0)) // Sim no mark
            .position_side(if new_qty > dec!(0) {
                PositionSide::Long
            } else {
                PositionSide::Short
            })
            .build();

        let au = AccountUpdate::builder()
            .event_time(self.time.now().await)
            .venue(self.venue.to_owned())
            .balances(vec![bal_update])
            .positions(vec![pos_update])
            .reason("ORDER".to_string())
            .build();
        self.publisher.publish(Event::AccountUpdate(au.to_owned().into())).await;
        info!(target: "executor-simulation", "published account update {}", au);
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn tick_update(&self, tick: &Tick) {
        debug!(target: "executor-simulation", "received tick update");

        for order in self.orderbook.list_orders() {
            if order.instrument.id == tick.instrument.id {
                info!(target: "executor-simulation", "checking order {}", order);
                info!(target: "executor-simulation", "best Bid {}, best ask {}", tick.bid_price(), tick.ask_price());
                let (matched, price, quantity, commission) = match order.order_type {
                    VenueOrderType::Market => self.check_market_order(tick, &order),
                    VenueOrderType::Limit => self.check_limit_order(tick, &order),
                    _ => {
                        warn!(target: "executor-simulation", "unsupported order type: {}", order.order_type);
                        continue;
                    }
                };

                info!(target: "executor-simulation", "order {} matched {}", order.id, matched);
                if matched {
                    self.fill_update(&order, price, quantity, commission).await;
                    self.account_update(&order, price, quantity, commission).await;
                }
            }
        }
    }
}

#[async_trait]
impl Runnable for Executor {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            Event::NewVenueOrder(o) => self.place_order(o).await,
            Event::CancelVenueOrder(o) => self.cancel_order(o).await,
            Event::CancelAllVenueOrders(_) => self.cancel_all().await,
            Event::TickUpdate(t) => self.tick_update(t).await,
            e => warn!(target: "executor-simulation", "received unused event {}", e),
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn setup(&self, _ctx: Arc<ServiceCtx>) {
        // Send inital balance update

        let mut balances = Vec::new();
        for entry in self.account.balances.iter() {
            let asset = entry.key();
            let balance = entry.value();
            balances.push(
                BalanceUpdate::builder()
                    .event_time(self.time.now().await)
                    .venue(test_binance_venue())
                    .asset(asset.to_owned())
                    .account_type(AccountType::Margin)
                    .quantity_change(balance.to_owned())
                    .quantity(balance.to_owned())
                    .build(),
            );
        }

        let mut positions = Vec::new();
        for entry in self.account.positions.iter() {
            let inst = entry.key();
            let (entry, qty) = entry.value();
            positions.push(
                PositionUpdate::builder()
                    .event_time(self.time.now().await)
                    .instrument(inst.to_owned())
                    .account_type(AccountType::Margin)
                    .entry_price(entry.to_owned())
                    .quantity(qty.to_owned())
                    .realized_pnl(dec!(0)) // Incremental; cumulate in full impl
                    .unrealized_pnl(dec!(0)) // Sim no mark
                    .position_side(if qty.to_owned() > dec!(0) {
                        PositionSide::Long
                    } else {
                        PositionSide::Short
                    })
                    .build(),
            );
        }
        let au = AccountUpdate::builder()
            .event_time(self.time.now().await)
            .venue(self.venue.to_owned())
            .balances(balances)
            .positions(positions)
            .reason("INIT".to_string())
            .build();
        self.publisher.publish(Event::InitialAccountUpdate(au.to_owned().into())).await;
        info!(target: "executor-simulation", "published initial account update {}", au);
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn start_tasks(self: Arc<Self>, _ctx: Arc<ServiceCtx>) {
        info!(target: "executor-simulation", "starting tasks");
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn stop_tasks(self: Arc<Self>, _ctx: Arc<ServiceCtx>) {
        info!(target: "executor-simulation", "stopping tasks");
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn teardown(&self, _ctx: Arc<ServiceCtx>) {
        info!(target: "executor-simulation", "teardown");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkin_core::test_utils::{MockPublisher, MockTime};
    use uuid::Uuid;

    #[tokio::test]
    #[test_log::test]
    async fn test_place_market_order() {
        // Setup
        let publisher = MockPublisher::new();
        let time = MockTime::new();
        let execution = Executor::new("test", time.clone(), publisher.clone());
        let mut order = VenueOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(VenueOrderType::Market)
            .side(MarketSide::Sell)
            .price(dec!(0))
            .quantity(dec!(1))
            .status(VenueOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();

        // Execute
        execution.handle_event(Event::NewVenueOrder(order.clone().into())).await;

        // Verify events
        let events = publisher.get_events().await;
        order.status = VenueOrderStatus::Inflight;
        assert_eq!(events.len(), 2, "Expected two events");
        assert_eq!(
            events[0],
            Event::VenueOrderInflight(order.clone().into()),
            "First event should be Inflight"
        );
        assert_eq!(execution.orderbook.len(), 1, "Order should be in the orderbook");

        // Check second event and orderbook state
        order.status = VenueOrderStatus::Placed;
        assert_eq!(
            events[1],
            Event::VenueOrderPlaced(order.clone().into()),
            "First event should be Inflight"
        );

        // Check orderbook
        assert_eq!(execution.orderbook.len(), 1, "Order should be in the orderbook");
        let order_in_book = execution.orderbook.get(order.id).unwrap();
        assert_eq!(order_in_book.status, VenueOrderStatus::Placed, "Status should be Placed");
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_place_limit_order() {
        // Setup
        let publisher = MockPublisher::new();
        let time = MockTime::new();
        let execution = Executor::new("test", time.clone(), publisher.clone());
        let mut order = VenueOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(VenueOrderType::Limit)
            .side(MarketSide::Buy)
            .price(dec!(100000))
            .quantity(dec!(1))
            .status(VenueOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();

        // Execute
        execution.handle_event(Event::NewVenueOrder(order.clone().into())).await;

        // Verify events
        let events = publisher.get_events().await;
        order.status = VenueOrderStatus::Inflight;
        assert_eq!(events.len(), 2, "Expected two events");
        assert_eq!(
            events[0],
            Event::VenueOrderInflight(order.clone().into()),
            "First event should be Inflight"
        );
        assert_eq!(execution.orderbook.len(), 1, "Order should be in the orderbook");

        // Check second event and orderbook state
        order.status = VenueOrderStatus::Placed;
        assert_eq!(
            events[1],
            Event::VenueOrderPlaced(order.clone().into()),
            "First event should be Inflight"
        );

        // Check orderbook
        assert_eq!(execution.orderbook.len(), 1, "Order should be in the orderbook");
        let order_in_book = execution.orderbook.get(order.id).unwrap();
        assert_eq!(order_in_book.status, VenueOrderStatus::Placed, "Status should be Placed");
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_cancel_order() {
        // Setup
        let publisher = MockPublisher::new();
        let time = MockTime::new();
        let execution = Executor::new("test", time.clone(), publisher.clone());
        let mut order = VenueOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(VenueOrderType::Limit)
            .side(MarketSide::Buy)
            .price(dec!(100000))
            .quantity(dec!(1))
            .status(VenueOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();

        // Place the order first
        execution.handle_event(Event::NewVenueOrder(order.clone().into())).await;
        let events = publisher.get_events().await;
        assert_eq!(events.len(), 2, "Expected two events from placing order");

        // Cancel the order
        execution.handle_event(Event::CancelVenueOrder(order.clone().into())).await;

        // Verify
        let events = publisher.get_events().await;
        order.status = VenueOrderStatus::Cancelled;
        assert_eq!(events.len(), 3, "Expected one event");
        assert_eq!(
            events[0],
            Event::VenueOrderCancelled(order.clone().into()),
            "Should publish Cancelled event"
        );
        assert_eq!(execution.orderbook.len(), 0, "Order should be removed");

        // Cancel the order
        execution.handle_event(Event::CancelVenueOrder(order.clone().into())).await;

        // Verify
        let events = publisher.get_events().await;
        // order.status = VenueOrderStatus::Cancelled;
        assert_eq!(events.len(), 3, "Expected one event");
        assert_eq!(execution.orderbook.len(), 0, "Order should be removed");
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_cancel_all() {
        // Setup
        let publisher = MockPublisher::new();
        let time = MockTime::new();
        let execution = Executor::new("test", time.clone(), publisher.clone());

        // Create and place three orders
        let order1 = VenueOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(VenueOrderType::Limit)
            .side(MarketSide::Buy)
            .price(dec!(101000))
            .quantity(dec!(1))
            .status(VenueOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();
        let order2 = VenueOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(VenueOrderType::Limit)
            .side(MarketSide::Buy)
            .price(dec!(100000))
            .quantity(dec!(1))
            .status(VenueOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();
        let order3 = VenueOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(VenueOrderType::Limit)
            .side(MarketSide::Buy)
            .price(dec!(99000))
            .quantity(dec!(1))
            .status(VenueOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();

        execution.handle_event(Event::NewVenueOrder(order1.clone().into())).await;
        execution.handle_event(Event::NewVenueOrder(order2.clone().into())).await;
        execution.handle_event(Event::NewVenueOrder(order3.clone().into())).await;

        // Verify that orders are in the orderbook
        assert_eq!(execution.orderbook.len(), 3, "Should have three orders in the orderbook");

        // Execute cancel_all
        execution.handle_event(Event::CancelAllVenueOrders(time.now().await)).await;

        // Verify orderbook is empty
        assert_eq!(execution.orderbook.len(), 0, "Orderbook should be empty after cancel_all");

        // Verify published events
        let events = publisher.get_events().await;
        let cancelled_events: Vec<_> = events
            .into_iter()
            .filter(|event| matches!(event, Event::VenueOrderCancelled(_)))
            .collect();

        assert_eq!(cancelled_events.len(), 3, "Should have three cancelled events");

        let cancelled_order_ids: Vec<Uuid> = cancelled_events
            .iter()
            .map(|event| {
                if let Event::VenueOrderCancelled(o) = event {
                    o.id
                } else {
                    unreachable!()
                }
            })
            .collect();

        assert!(cancelled_order_ids.contains(&order1.id), "Order1 should be cancelled");
        assert!(cancelled_order_ids.contains(&order2.id), "Order2 should be cancelled");
        assert!(cancelled_order_ids.contains(&order3.id), "Order3 should be cancelled");
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_cancel_all_empty() {
        // Setup
        let publisher = MockPublisher::new();
        let time = MockTime::new();
        let execution = Executor::new("test", time.clone(), publisher.clone());

        // Execute cancel_all on empty orderbook
        execution.handle_event(Event::CancelAllVenueOrders(time.now().await)).await;

        // Verify no events are published
        let events = publisher.get_events().await;
        assert_eq!(
            events.len(),
            0,
            "No events should be published when cancelling all on empty orderbook"
        );

        // Verify orderbook is still empty
        assert_eq!(execution.orderbook.len(), 0, "Orderbook should remain empty");
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_tick_update_market_orders() {
        // Setup
        let publisher = MockPublisher::new();
        let time = MockTime::new();
        let execution = Executor::new("test", time.clone(), publisher.clone());

        // Create buy and sell market orders
        let mut buy_order = VenueOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(VenueOrderType::Market)
            .side(MarketSide::Buy)
            .price(dec!(0))
            .quantity(dec!(1))
            .status(VenueOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();
        let mut sell_order = VenueOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(VenueOrderType::Market)
            .side(MarketSide::Sell)
            .price(dec!(0))
            .quantity(dec!(1))
            .status(VenueOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();

        // Place the buy order
        execution.handle_event(Event::NewVenueOrder(buy_order.clone().into())).await;
        // Place the sell order
        execution.handle_event(Event::NewVenueOrder(sell_order.clone().into())).await;

        // Verify initial events after placing orders (2 events per order: Inflight and Placed)
        let events = publisher.get_events().await;
        assert_eq!(events.len(), 4, "Expected four events from placing two orders");

        buy_order.status = VenueOrderStatus::Inflight;
        assert_eq!(
            events[0],
            Event::VenueOrderInflight(buy_order.clone().into()),
            "Buy order: First event should be Inflight"
        );
        buy_order.status = VenueOrderStatus::Placed;
        assert_eq!(
            events[1],
            Event::VenueOrderPlaced(buy_order.clone().into()),
            "Buy order: Second event should be Placed"
        );

        sell_order.status = VenueOrderStatus::Inflight;
        assert_eq!(
            events[2],
            Event::VenueOrderInflight(sell_order.clone().into()),
            "Sell order: First event should be Inflight"
        );
        sell_order.status = VenueOrderStatus::Placed;
        assert_eq!(
            events[3],
            Event::VenueOrderPlaced(sell_order.clone().into()),
            "Sell order: Second event should be Placed"
        );

        // Verify orderbook state
        assert_eq!(execution.orderbook.len(), 2, "Both orders should be in the orderbook");

        // Create a tick
        let tick = Tick::builder()
            .event_time(time.now().await)
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(0 as u64)
            .bid_price(dec!(49000.0))
            .bid_quantity(dec!(0.7))
            .ask_price(dec!(50000.0))
            .ask_quantity(dec!(0.3))
            .build();

        // Send tick update
        execution.handle_event(Event::TickUpdate(tick.clone().into())).await;

        // Verify events after tick update (additional 2 Filled events)
        let events = publisher.get_events().await;
        assert_eq!(
            events.len(),
            8,
            "Expected six events total (4 from placing, 2 from fill updates, 2 from account updates)"
        );

        // Check buy order filled event
        buy_order.status = VenueOrderStatus::PartiallyFilled;
        buy_order.filled_price = dec!(50000.0); // Filled at ask price
        buy_order.filled_quantity = tick.bid_quantity;
        assert_eq!(
            events[4],
            Event::VenueOrderFill(buy_order.clone().into()),
            "Buy order should be partially filled at ask price"
        );

        // Check sell order filled event
        sell_order.status = VenueOrderStatus::PartiallyFilled;
        sell_order.filled_price = dec!(49000.0); // Filled at bid price
        sell_order.filled_quantity = tick.ask_quantity;
        assert_eq!(
            events[6],
            Event::VenueOrderFill(sell_order.clone().into()),
            "Sell order should be partially filled at bid price"
        );

        // Verify orderbook is empty
        assert_eq!(
            execution.orderbook.len(),
            2,
            "Orderbook should have two partially filled orders"
        );

        // Create a tick
        let tick = Tick::builder()
            .event_time(time.now().await)
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(0 as u64)
            .bid_price(dec!(50000.0))
            .bid_quantity(dec!(0.3))
            .ask_price(dec!(51000.0))
            .ask_quantity(dec!(0.7))
            .build();

        // Send tick update
        execution.handle_event(Event::TickUpdate(tick.clone().into())).await;

        // Verify events after tick update (additional 2 Filled events)
        let events = publisher.get_events().await;
        assert_eq!(
            events.len(),
            12,
            "Expected six events total (4 from placing, 4 from filling, 4 from account updates)"
        );

        // Check buy order filled event
        buy_order.status = VenueOrderStatus::Filled;
        buy_order.filled_price = dec!(50000.0); // Filled at ask price
        buy_order.filled_quantity = tick.bid_quantity;
        assert_eq!(
            events[8],
            Event::VenueOrderFill(buy_order.clone().into()),
            "Buy order should be Filled at ask price"
        );

        // Check sell order filled event
        sell_order.status = VenueOrderStatus::Filled;
        sell_order.filled_price = dec!(49000.0); // Filled at bid price
        sell_order.filled_quantity = tick.ask_quantity;
        assert_eq!(
            events[10],
            Event::VenueOrderFill(sell_order.clone().into()),
            "Sell order should be Filled at bid price"
        );

        // Verify orderbook is empty
        assert_eq!(
            execution.orderbook.len(),
            0,
            "Orderbook should have two partially filled orders"
        );
    }

    #[tokio::test]
    #[test_log::test]
    async fn test_tick_update_limit_orders() {
        // Setup
        let publisher = MockPublisher::new();
        let time = MockTime::new();
        let execution = Executor::new("test", time.clone(), publisher.clone());
        // Create buy and sell limit orders (Will fill at ask 49500)
        let mut buy_order = VenueOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(VenueOrderType::Limit)
            .side(MarketSide::Buy)
            .price(dec!(49000.0))
            .quantity(dec!(1))
            .status(VenueOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();
        let mut sell_order = VenueOrder::builder()
            .id(Uuid::new_v4())
            .strategy(Some(test_strategy_1()))
            .instrument(test_inst_binance_btc_usdt_perp())
            .order_type(VenueOrderType::Limit)
            .side(MarketSide::Sell)
            .price(dec!(49000.0))
            .quantity(dec!(1))
            .status(VenueOrderStatus::New)
            .created_at(time.now().await)
            .updated_at(time.now().await)
            .build();

        // Place the buy order
        execution.handle_event(Event::NewVenueOrder(buy_order.clone().into())).await;
        // Place the sell order
        execution.handle_event(Event::NewVenueOrder(sell_order.clone().into())).await;

        // Verify initial events after placing orders (2 events per order: Inflight and Placed)
        let events = publisher.get_events().await;
        assert_eq!(events.len(), 4, "Expected four events from placing two orders");

        buy_order.status = VenueOrderStatus::Inflight;
        assert_eq!(
            events[0],
            Event::VenueOrderInflight(buy_order.clone().into()),
            "Buy order: First event should be Inflight"
        );
        buy_order.status = VenueOrderStatus::Placed;
        assert_eq!(
            events[1],
            Event::VenueOrderPlaced(buy_order.clone().into()),
            "Buy order: Second event should be Placed"
        );

        sell_order.status = VenueOrderStatus::Inflight;
        assert_eq!(
            events[2],
            Event::VenueOrderInflight(sell_order.clone().into()),
            "Sell order: First event should be Inflight"
        );
        sell_order.status = VenueOrderStatus::Placed;
        assert_eq!(
            events[3],
            Event::VenueOrderPlaced(sell_order.clone().into()),
            "Sell order: Second event should be Placed"
        );

        // Verify orderbook state
        assert_eq!(execution.orderbook.len(), 2, "Both orders should be in the orderbook");

        // Create a tick where buy limit (49500) >= ask (49500) and sell limit (49000) <= bid (49500)
        let tick = Tick::builder()
            .event_time(time.now().await)
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(0 as u64)
            .bid_price(dec!(49400.0))
            .bid_quantity(dec!(1))
            .ask_price(dec!(49600.0))
            .ask_quantity(dec!(1))
            .build();

        // Send tick update
        execution.handle_event(Event::TickUpdate(tick.into())).await;

        // Verify events after tick update (additional 2 Filled events)
        let events = publisher.get_events().await;
        assert_eq!(
            events.len(),
            6,
            "Expected six events total (4 from placing, 1 from filling, 1 account update)"
        );

        // Check buy order filled event
        buy_order.status = VenueOrderStatus::Filled;
        buy_order.filled_price = dec!(49400.0); // Filled at ask price
        buy_order.filled_quantity = buy_order.quantity;
        assert_eq!(
            events[5],
            Event::VenueOrderFill(buy_order.clone().into()),
            "Buy order should be Filled at ask price"
        );

        // Create a tick where buy limit (49500) >= ask (49500) and sell limit (49000) <= bid (49500)
        let tick = Tick::builder()
            .event_time(time.now().await)
            .instrument(test_inst_binance_btc_usdt_perp())
            .tick_id(1 as u64)
            .bid_price(dec!(48700.0))
            .bid_quantity(dec!(1))
            .ask_price(dec!(48900.0))
            .ask_quantity(dec!(1))
            .build();

        // Send tick update
        execution.handle_event(Event::TickUpdate(tick.into())).await;

        // Verify events after tick update (additional 2 Filled events)
        let events = publisher.get_events().await;
        assert_eq!(
            events.len(),
            8,
            "Expected six events total (4 from placing, 2 from filling, 2 account updates)"
        );

        // Check sell order filled event
        sell_order.status = VenueOrderStatus::Filled;
        sell_order.filled_price = dec!(48900.0); // Filled at bid price
        sell_order.filled_quantity = sell_order.quantity;
        assert_eq!(
            events[7],
            Event::VenueOrderFill(sell_order.clone().into()),
            "Sell order should be Filled at bid price"
        );

        // Verify orderbook is empty
        assert_eq!(
            execution.orderbook.len(),
            0,
            "Orderbook should be empty after both orders are filled"
        );
    }
}
