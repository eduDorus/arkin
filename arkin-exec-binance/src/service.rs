use std::sync::Arc;
use std::time::Duration;

use arkin_core::prelude::*;
use async_trait::async_trait;
use tokio::time::timeout;
use tokio_util::task::TaskTracker;
use tracing::{error, info, warn};
use typed_builder::TypedBuilder;

use crate::{client::BinanceClient, types::BinanceMarketType};
const ORDER_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RETRIES: u32 = 3;

#[derive(TypedBuilder)]
pub struct BinanceExecution {
    client: Arc<BinanceClient>,
    #[builder(default = TaskTracker::new())]
    task_tracker: TaskTracker,
}

pub type BinanceExecutionService = BinanceExecution;

impl BinanceExecution {
    pub fn from_config() -> Arc<Self> {
        let client = Arc::new(BinanceClient::from_config());
        Self {
            client,
            task_tracker: TaskTracker::new(),
        }
        .into()
    }

    /// Gracefully shutdown, waiting for all in-flight order requests to complete
    pub async fn shutdown(&self) {
        info!("Shutting down BinanceExecution, waiting for in-flight requests...");
        self.task_tracker.close();
        self.task_tracker.wait().await;
        info!("All order requests completed");
    }

    async fn handle_new_venue_order(&self, ctx: Arc<CoreCtx>, order: Arc<VenueOrder>) {
        if !matches!(order.instrument.venue.name, VenueName::Binance) {
            warn!(
                "Order {} for wrong venue: {}, expected Binance",
                order.id, order.instrument.venue.name
            );
            return;
        }

        let client = Arc::clone(&self.client);
        let ctx_clone = Arc::clone(&ctx);
        let order_clone = Arc::clone(&order);

        // Track spawned task for graceful shutdown
        self.task_tracker.spawn(async move {
            info!("Placing venue order: {}", order_clone.id);

            // Retry logic with timeout
            let mut attempts = 0u32;
            loop {
                attempts += 1;

                match timeout(ORDER_TIMEOUT, client.place_order(&order_clone)).await {
                    Ok(Ok(response)) => {
                        info!(
                            "Successfully placed order {} with Binance ID: {} (attempt {})",
                            order_clone.id, response.order_id, attempts
                        );
                        // Don't publish VenueOrderPlaced - user data stream handles this
                        return;
                    }
                    Ok(Err(e)) => {
                        if attempts >= MAX_RETRIES {
                            error!("Failed to place order {} after {} attempts: {}", order_clone.id, attempts, e);
                            // Publish rejection so system knows order failed
                            let venue_order_update = VenueOrderUpdate::builder()
                                .event_time(ctx_clone.now().await)
                                .id(order_clone.id)
                                .status(VenueOrderStatus::Rejected)
                                .build();
                            let _ = ctx_clone.publish(Event::VenueOrderUpdate(venue_order_update.into())).await;
                            return;
                        }
                        warn!(
                            "Failed to place order {} (attempt {}): {}, retrying...",
                            order_clone.id, attempts, e
                        );
                        tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                    }
                    Err(_) => {
                        if attempts >= MAX_RETRIES {
                            error!("Order {} timed out after {} attempts", order_clone.id, attempts);
                            let venue_order_update = VenueOrderUpdate::builder()
                                .event_time(ctx_clone.now().await)
                                .id(order_clone.id)
                                .status(VenueOrderStatus::Rejected)
                                .build();
                            let _ = ctx_clone.publish(Event::VenueOrderUpdate(venue_order_update.into())).await;
                            return;
                        }
                        warn!("Order {} timed out (attempt {}), retrying...", order_clone.id, attempts);
                        tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                    }
                }
            }
        });
    }

    async fn handle_cancel_venue_order(&self, _ctx: Arc<CoreCtx>, order: Arc<VenueOrder>) {
        if !matches!(order.instrument.venue.name, VenueName::Binance) {
            warn!(
                "Cancel request for {} for wrong venue: {}, expected Binance",
                order.id, order.instrument.venue.name
            );
            return;
        }

        let client = Arc::clone(&self.client);
        let order_clone = Arc::clone(&order);

        // Track spawned task for graceful shutdown
        self.task_tracker.spawn(async move {
            info!("Canceling venue order: {}", order_clone.id);

            // Retry logic with timeout
            let mut attempts = 0u32;
            loop {
                attempts += 1;

                match timeout(ORDER_TIMEOUT, client.cancel_order(&order_clone)).await {
                    Ok(Ok(response)) => {
                        info!(
                            "Successfully cancelled order {} with Binance ID: {} (attempt {})",
                            order_clone.id, response.order_id, attempts
                        );
                        // Don't publish VenueOrderCancelled - user data stream handles this
                        return;
                    }
                    Ok(Err(e)) => {
                        let error_msg = e.to_string();

                        // Check if order was already filled/cancelled (not an error)
                        if error_msg.contains("-2011") || error_msg.contains("Unknown order") {
                            info!("Order {} no longer exists (likely filled/cancelled) - ignoring", order_clone.id);
                            return;
                        }

                        if attempts >= MAX_RETRIES {
                            error!("Failed to cancel order {} after {} attempts: {}", order_clone.id, attempts, e);
                            return;
                        }
                        warn!(
                            "Failed to cancel order {} (attempt {}): {}, retrying...",
                            order_clone.id, attempts, e
                        );
                        tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                    }
                    Err(_) => {
                        if attempts >= MAX_RETRIES {
                            error!("Cancel request for {} timed out after {} attempts", order_clone.id, attempts);
                            return;
                        }
                        warn!(
                            "Cancel request for {} timed out (attempt {}), retrying...",
                            order_clone.id, attempts
                        );
                        tokio::time::sleep(Duration::from_millis(100 * attempts as u64)).await;
                    }
                }
            }
        });
    }

    async fn handle_cancel_all_venue_orders(&self, _ctx: Arc<CoreCtx>, symbol: Option<String>) {
        let client = Arc::clone(&self.client);

        self.task_tracker.spawn(async move {
            info!("Canceling all venue orders for symbol: {:?}", symbol);

            // Cancel on both spot and futures markets
            let spot_result = client.cancel_all_orders(symbol.as_deref(), BinanceMarketType::Spot).await;
            let usdm_result = client.cancel_all_orders(symbol.as_deref(), BinanceMarketType::Usdm).await;

            match (spot_result, usdm_result) {
                (Ok(_), Ok(_)) => {
                    info!("Successfully cancelled all orders on both markets");
                }
                (Ok(_), Err(e)) => {
                    warn!("Cancelled spot orders but failed to cancel USDM orders: {}", e);
                }
                (Err(e), Ok(_)) => {
                    warn!("Cancelled USDM orders but failed to cancel spot orders: {}", e);
                }
                (Err(e1), Err(e2)) => {
                    error!("Failed to cancel orders on both markets: spot={}, usdm={}", e1, e2);
                }
            }
        });
    }
}

#[async_trait]
impl Runnable for BinanceExecution {
    fn event_filter(&self, _instance_type: InstanceType) -> EventFilter {
        EventFilter::Events(vec![
            EventType::NewVenueOrder,
            EventType::CancelVenueOrder,
            EventType::CancelAllVenueOrders,
        ])
    }

    async fn handle_event(&self, core_ctx: Arc<CoreCtx>, event: Event) {
        match event {
            Event::NewVenueOrder(order) => {
                self.handle_new_venue_order(core_ctx, order).await;
            }
            Event::CancelVenueOrder(order) => {
                self.handle_cancel_venue_order(core_ctx, order).await;
            }
            Event::CancelAllVenueOrders(_time) => {
                self.handle_cancel_all_venue_orders(core_ctx, None).await;
            }
            _ => {}
        }
    }
}
