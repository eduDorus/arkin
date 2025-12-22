use std::sync::Arc;

use arkin_core::prelude::*;
use async_trait::async_trait;
use tokio::task;
use tracing::{error, info, warn};
use typed_builder::TypedBuilder;

use crate::{
    client::BinanceClient,
    config::{BinanceExecutionConfig, BinanceExecutionServiceConfig},
    types::BinanceMarketType,
};

#[derive(TypedBuilder)]
pub struct BinanceExecution {
    client: Arc<BinanceClient>,
}

pub type BinanceExecutionService = BinanceExecution;

impl BinanceExecution {
    pub fn new(config: BinanceExecutionServiceConfig) -> Arc<Self> {
        Self::new_with_clients(config.binance_execution, None, None)
    }

    pub fn new_with_clients(
        config: BinanceExecutionConfig,
        spot_client: Option<reqwest::Client>,
        usdm_client: Option<reqwest::Client>,
    ) -> Arc<Self> {
        let client = Arc::new(BinanceClient::new_with_clients(config, spot_client, usdm_client));
        Self { client }.into()
    }

    pub fn from_config() -> Arc<Self> {
        let client = Arc::new(BinanceClient::from_config());
        Self { client }.into()
    }

    async fn handle_new_venue_order(&self, ctx: Arc<CoreCtx>, order: Arc<VenueOrder>) {
        info!("Handling new venue order: {}", order.id);
        if !matches!(order.instrument.venue.name, VenueName::Binance) {
            warn!(
                "Received order for different venue: {}, expected: {}",
                order.instrument.venue.name,
                VenueName::Binance
            );
            return; // Not for this venue
        }

        let client = Arc::clone(&self.client);
        let ctx_clone = Arc::clone(&ctx);
        let order_clone = Arc::clone(&order);

        task::spawn(async move {
            info!("Placing venue order: {}", order_clone.id);

            match client.place_order(&order_clone).await {
                Ok(response) => {
                    info!(
                        "Successfully placed order {} with Binance ID: {}",
                        order_clone.id, response.order_id
                    );

                    // Update the order with venue order ID
                    // Note: In a real implementation, you'd need to update the VenueOrderBook
                    // For now, we'll just publish the placed event
                    let _ = ctx_clone.publish(Event::VenueOrderPlaced(order_clone)).await;
                }
                Err(e) => {
                    error!("Failed to place order {}: {}", order_clone.id, e);
                    // Publish failed event if needed
                }
            }
        });
    }

    async fn handle_cancel_venue_order(&self, ctx: Arc<CoreCtx>, order: Arc<VenueOrder>) {
        info!("Handling cancel venue order: {}", order.id);
        if !matches!(order.instrument.venue.name, VenueName::Binance) {
            warn!(
                "Received order for different venue: {}, expected: {}",
                order.instrument.venue.name,
                VenueName::Binance
            );
            return; // Not for this venue
        }

        let client = Arc::clone(&self.client);
        let ctx_clone = Arc::clone(&ctx);
        let order_clone = Arc::clone(&order);

        task::spawn(async move {
            info!("Canceling venue order: {}", order_clone.id);

            match client.cancel_order(&order_clone).await {
                Ok(response) => {
                    info!(
                        "Successfully cancelled order {} with Binance ID: {}",
                        order_clone.id, response.order_id
                    );
                    let _ = ctx_clone.publish(Event::VenueOrderCancelled(order_clone)).await;
                }
                Err(e) => {
                    error!("Failed to cancel order {}: {}", order_clone.id, e);
                }
            }
        });
    }

    async fn handle_cancel_all_venue_orders(&self, _ctx: Arc<CoreCtx>, symbol: Option<String>) {
        let client = Arc::clone(&self.client);

        task::spawn(async move {
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
        info!("BinanceExecution handling event: {}", event);
        match event {
            Event::NewVenueOrder(order) => {
                self.handle_new_venue_order(core_ctx, order).await;
            }
            Event::CancelVenueOrder(order) => {
                self.handle_cancel_venue_order(core_ctx, order).await;
            }
            Event::CancelAllVenueOrders(_time) => {
                // For cancel all, we don't have symbol filtering in the event
                // In a real implementation, you might want to filter by venue
                self.handle_cancel_all_venue_orders(core_ctx, None).await;
            }
            _ => {}
        }
    }
}
