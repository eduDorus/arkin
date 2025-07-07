use std::sync::Arc;

use async_trait::async_trait;
use dashmap::DashMap;
use tracing::{error, info, instrument, warn};

use arkin_core::prelude::*;
use uuid::Uuid;

#[derive(Default)]
pub struct OrderBook {
    queue: DashMap<Uuid, VenueOrder>,
}

impl OrderBook {
    pub fn insert(&self, order: VenueOrder) {
        if !matches!(order.status, VenueOrderStatus::Inflight) {
            error!(target: "simulation-executor", "Adding order to order book that is not inflight");
        }
        self.queue.insert(order.id, order);
    }

    pub fn get(&self, id: Uuid) -> Option<VenueOrder> {
        self.queue.get(&id).map(|entry| entry.value().clone())
    }

    pub fn update(&self, order: VenueOrder) {
        if let Some(mut o) = self.queue.get_mut(&order.id) {
            *o = order;
        } else {
            error!(target: "simulation-executor", "Updating order that does not exist in the order book");
        }
    }

    pub fn remove(&self, id: Uuid) -> Option<(Uuid, VenueOrder)> {
        self.queue.remove(&id)
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn list_orders(&self) -> Vec<VenueOrder> {
        let mut orders: Vec<VenueOrder> = self.queue.iter().map(|entry| entry.value().clone()).collect();
        orders.sort_by_key(|order| order.status);
        orders
    }

    pub fn list_orders_by_status(&self, status: VenueOrderStatus) -> Vec<VenueOrder> {
        self.queue
            .iter()
            .filter(|entry| entry.value().status == status)
            .map(|entry| entry.value().clone())
            .collect()
    }
}

pub struct SimulationExecution {
    identifier: String,
    time: Arc<dyn SystemTime>,
    publisher: Arc<dyn Publisher>,
    orderbook: OrderBook,
}

impl SimulationExecution {
    pub fn new(identifier: &str, time: Arc<dyn SystemTime>, publisher: Arc<dyn Publisher>) -> Arc<Self> {
        Self {
            identifier: identifier.to_owned(),
            time,
            publisher,
            orderbook: OrderBook::default(),
        }
        .into()
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn place_order(&self, order: &VenueOrder) {
        info!(target: "executor", "received new order");

        info!(target: "executor", "change order to inflight and add order {} to orderbook", order.id);
        let mut order = order.clone();
        let time = self.time.now().await;
        order.update_status(VenueOrderStatus::Inflight, time);
        self.orderbook.insert(order.clone());
        self.publisher.publish(Event::VenueOrderInflight(order.clone().into())).await;

        info!(target: "executor", "change order to placed and sending placed event for order {}", order.id);
        let time = self.time.now().await;
        order.update_status(VenueOrderStatus::Placed, time);
        self.orderbook.update(order.clone());
        self.publisher.publish(Event::VenueOrderPlaced(order.into())).await;
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn cancel_order(&self, order: &VenueOrder) {
        info!(target: "executor", "received cancel order");

        if let Some((_, order)) = self.orderbook.remove(order.id) {
            info!(target: "executor", "order {} successfully cancelled", order.id);
            self.publisher.publish(Event::VenueOrderCancelled(order.into())).await;
        } else {
            warn!(target: "executor", "order {} not in order book, could not cancel", order.id)
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn cancel_all(&self) {
        info!(target: "executor", "received cancel all orders");

        let orders = self.orderbook.list_orders();
        for order in orders {
            if let Some((_, order)) = self.orderbook.remove(order.id) {
                self.publisher.publish(Event::VenueOrderCancelled(order.into())).await;
            } else {
                warn!(target: "executor", "order {} not in order book, could not cancel", order.id)
            }
        }
    }
}

#[async_trait]
impl Runnable for SimulationExecution {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            Event::NewVenueOrder(o) => self.place_order(o).await,
            Event::CancelVenueOrder(o) => self.cancel_order(o).await,
            Event::CancelAllVenueOrders(_) => self.cancel_all().await,
            e => warn!(target: "executor", "received unused event {}", e),
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn start_tasks(self: Arc<Self>, _ctx: Arc<ServiceCtx>) {
        info!(target: "executor", "starting tasks");
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn stop_tasks(self: Arc<Self>, _ctx: Arc<ServiceCtx>) {
        info!(target: "executor", "stopping tasks");
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn teardown(&self, _ctx: Arc<ServiceCtx>) {
        info!(target: "executor", "teardown");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkin_core::test_utils::{MockPublisher, MockTime};

    #[tokio::test]
    #[test_log::test]
    async fn test_place_order() {
        // Setup
        let publisher = MockPublisher::new();
        let time = MockTime::new();
        let execution = SimulationExecution::new("test", time.clone(), publisher.clone());
        let mut order = test_venue_order_new(time.now().await);

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
        let execution = SimulationExecution::new("test", time.clone(), publisher.clone());
        let mut order = test_venue_order_new(time.now().await);

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

    #[cfg(test)]
    mod tests {
        use super::*;
        use arkin_core::test_utils::{MockPublisher, MockTime};

        #[tokio::test]
        #[test_log::test]
        async fn test_cancel_all() {
            // Setup
            let publisher = MockPublisher::new();
            let time = MockTime::new();
            let execution = SimulationExecution::new("test", time.clone(), publisher.clone());

            // Create and place three orders
            let order1 = test_venue_order_new(time.now().await);
            let order2 = test_venue_order_new(time.now().await);
            let order3 = test_venue_order_new(time.now().await);

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
            let execution = SimulationExecution::new("test", time.clone(), publisher.clone());

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
    }
}
