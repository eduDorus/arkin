use std::sync::Arc;

use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{OrderManager, OrderManagerError, OrderManagerService};

#[derive(Debug, TypedBuilder)]
pub struct DefaultOrderManager {
    pubsub: Arc<PubSub>,
}

#[async_trait]
impl OrderManagerService for DefaultOrderManager {}

#[async_trait]
impl OrderManager for DefaultOrderManager {
    async fn place_order(&self, order: Arc<ExecutionOrder>) -> Result<(), OrderManagerError> {
        match order.order_type {
            ExecutionOrderType::Taker => {
                info!("Placing market order: {}", order);
                let venue_order = VenueOrder::builder()
                    .id(order.id)
                    .strategy(test_strategy())
                    .instrument(order.instrument.clone())
                    .side(order.side)
                    .quantity(order.quantity)
                    .price(order.price)
                    .order_type(VenueOrderType::Market)
                    .build();
                self.pubsub.publish(Event::VenueOrderNew(venue_order.into())).await;
            }
            ExecutionOrderType::Maker => {
                info!("Placing limit order: {}", order);
                let venue_order = VenueOrder::builder()
                    .id(order.id)
                    .strategy(test_strategy())
                    .instrument(order.instrument.clone())
                    .side(order.side)
                    .quantity(order.quantity)
                    .price(order.price)
                    .order_type(VenueOrderType::Limit)
                    .build();
                self.pubsub.publish(Event::VenueOrderNew(venue_order.into())).await;
            }
            _ => {
                warn!("Unsupported order type: {}", order);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl RunnableService for DefaultOrderManager {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting order manager...");

        let mut rx = self.pubsub.subscribe();

        loop {
            tokio::select! {
                Ok(event) = rx.recv() => {
                  match event {
                    Event::ExecutionOrderNew(order) => {
                      info!("SimpleOrderManager received order: {}", order);
                      self.place_order(order).await.unwrap();
                    }
                    Event::VenueOrderUpdate(order) => {
                      info!("SimpleOrderManager received order update: {}", order);
                    }
                    _ => {}
                  }
                }
                _ = shutdown.cancelled() => {
                    info!("Execution shutdown...");
                    break;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use rust_decimal_macros::dec;
    use test_log::test;
    use tokio::time::timeout;

    #[test(tokio::test)]
    async fn taker_strategy() {
        let pubsub = PubSub::new(1024);
        let order_manager = DefaultOrderManager::builder().pubsub(pubsub.clone()).build();

        let order = ExecutionOrder::builder()
            .id(VenueOrderId::new_v4())
            .portfolio(test_portfolio())
            .instrument(test_inst_binance_btc_usdt_perp())
            .side(MarketSide::Buy)
            .quantity(dec!(1.0))
            .price(dec!(50000.0))
            .order_type(ExecutionOrderType::Taker)
            .build();
        let order_arc = Arc::new(order);

        let mut rx = pubsub.subscribe();

        order_manager.place_order(order_arc.clone()).await.unwrap();

        // Verify that a VenueOrderUpdate event was published.
        // We use a timeout in case no event is published.
        let event = timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("Expected event within 1 sec")
            .expect("Event channel closed");
        match event {
            Event::VenueOrderNew(new_order) => {
                assert_eq!(new_order.id, order_arc.id);
                assert_eq!(new_order.status, VenueOrderStatus::New);
                assert_eq!(new_order.side, MarketSide::Buy);
                assert_eq!(new_order.order_type, VenueOrderType::Market);
            }
            _ => panic!("Expected VenueOrderNew event"),
        }
    }

    #[test(tokio::test)]
    async fn maker_strategy() {
        let pubsub = PubSub::new(1024);
        let order_manager = DefaultOrderManager::builder().pubsub(pubsub.clone()).build();

        let order = ExecutionOrder::builder()
            .id(VenueOrderId::new_v4())
            .portfolio(test_portfolio())
            .instrument(test_inst_binance_btc_usdt_perp())
            .side(MarketSide::Buy)
            .quantity(dec!(1.0))
            .price(dec!(50000.0))
            .order_type(ExecutionOrderType::Maker)
            .build();
        let order_arc = Arc::new(order);

        let mut rx = pubsub.subscribe();

        order_manager.place_order(order_arc.clone()).await.unwrap();

        // Verify that a VenueOrderUpdate event was published.
        // We use a timeout in case no event is published.
        let event = timeout(Duration::from_secs(1), rx.recv())
            .await
            .expect("Expected event within 1 sec")
            .expect("Event channel closed");
        match event {
            Event::VenueOrderNew(new_order) => {
                assert_eq!(new_order.id, order_arc.id);
                assert_eq!(new_order.status, VenueOrderStatus::New);
                assert_eq!(new_order.side, MarketSide::Buy);
                assert_eq!(new_order.order_type, VenueOrderType::Limit);
            }
            _ => panic!("Expected VenueOrderNew event"),
        }
    }
}
