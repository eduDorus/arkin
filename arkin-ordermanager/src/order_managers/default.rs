use std::sync::Arc;

use async_trait::async_trait;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{OrderManager, OrderManagerError, OrderManagerService};

#[derive(TypedBuilder)]
pub struct DefaultOrderManager {
    pubsub: PubSubHandle,
}

#[async_trait]
impl OrderManagerService for DefaultOrderManager {}

#[async_trait]
impl OrderManager for DefaultOrderManager {
    async fn place_order(&self, order: Arc<ExecutionOrder>) -> Result<(), OrderManagerError> {
        info!("DefaultOrderManager received order: {}", order);

        match order.order_type {
            ExecutionOrderType::Taker => {
                info!("DefaultOrderManager placing market order: {}", order);
                let venue_order = VenueOrder::builder()
                    .id(order.id)
                    .event_time(order.event_time)
                    .strategy(order.strategy.clone().unwrap())
                    .instrument(order.instrument.clone())
                    .side(order.side)
                    .quantity(order.quantity)
                    .price(order.price)
                    .order_type(VenueOrderType::Market)
                    .updated_at(order.event_time)
                    .build();
                self.pubsub.publish(Event::VenueOrderNew(venue_order.into())).await;
            }
            ExecutionOrderType::Maker => {
                info!("DefaultOrderManager placing limit order: {}", order);
                let venue_order = VenueOrder::builder()
                    .id(order.id)
                    .event_time(order.event_time)
                    .strategy(order.strategy.clone().unwrap())
                    .instrument(order.instrument.clone())
                    .side(order.side)
                    .quantity(order.quantity)
                    .price(order.price)
                    .order_type(VenueOrderType::Limit)
                    .updated_at(order.event_time)
                    .build();
                self.pubsub.publish(Event::VenueOrderNew(venue_order.into())).await;
            }
            _ => {
                warn!("Unsupported order type: {}", order);
            }
        }
        Ok(())
    }

    async fn order_update(&self, _fill: Arc<VenueOrder>) -> Result<(), OrderManagerError> {
        info!("DefaultOrderManager received order update: {}", _fill);
        Ok(())
    }

    async fn order_fill(&self, _fill: Arc<VenueOrder>) -> Result<(), OrderManagerError> {
        info!("DefaultOrderManager received order fill: {}", _fill);
        Ok(())
    }
}

#[async_trait]
impl RunnableService for DefaultOrderManager {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), anyhow::Error> {
        info!("Starting order manager...");

        loop {
            tokio::select! {
                Ok((event, barrier)) = self.pubsub.rx.recv() => {
                  info!("OrderManager received event");
                  match event {
                    Event::ExecutionOrderNew(order) => self.place_order(order).await?,
                    Event::VenueOrderStatusUpdate(order) => self.order_update(order).await?,
                    Event::VenueOrderFillUpdate(order) => self.order_fill(order).await?,
                    Event::Finished => {
                      barrier.wait().await;
                      break;
                  }
                    _ => {}
                  }
                  info!("OrderManager event processed");
                  barrier.wait().await;
                }
                _ = shutdown.cancelled() => {
                    info!("Execution shutdown...");
                    break;
                }
            }
        }
        info!("Order manager stopped.");
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::time::Duration;

//     use super::*;
//     use rust_decimal_macros::dec;
//     use test_log::test;
//     use time::OffsetDateTime;
//     use tokio::time::timeout;

//     #[test(tokio::test)]
//     async fn taker_strategy() {
//         let pubsub = PubSub::default();
//         let order_manager = DefaultOrderManager::builder().pubsub(pubsub.handle().await).build();

//         let order = ExecutionOrder::builder()
//             .id(VenueOrderId::new_v4())
//             .event_time(OffsetDateTime::now_utc())
//             .strategy(Some(test_strategy_1()))
//             .instrument(test_inst_binance_btc_usdt_perp())
//             .side(MarketSide::Buy)
//             .quantity(dec!(1.0))
//             .price(dec!(50000.0))
//             .order_type(ExecutionOrderType::Taker)
//             .build();
//         let order_arc = Arc::new(order);

//         let mut rx = pubsub.subscribe();

//         order_manager.place_order(order_arc.clone()).await.unwrap();

//         // Verify that a VenueOrderUpdate event was published.
//         // We use a timeout in case no event is published.
//         let event = timeout(Duration::from_secs(1), rx.recv())
//             .await
//             .expect("Expected event within 1 sec")
//             .expect("Event channel closed");
//         match event {
//             Event::VenueOrderNew(new_order) => {
//                 assert_eq!(new_order.id, order_arc.id);
//                 assert_eq!(new_order.status, VenueOrderStatus::New);
//                 assert_eq!(new_order.side, MarketSide::Buy);
//                 assert_eq!(new_order.order_type, VenueOrderType::Market);
//             }
//             _ => panic!("Expected VenueOrderNew event"),
//         }
//     }

//     #[test(tokio::test)]
//     async fn maker_strategy() {
//         let pubsub = PubSub::default();
//         let order_manager = DefaultOrderManager::builder().pubsub(pubsub.handle().await).build();

//         let order = ExecutionOrder::builder()
//             .id(VenueOrderId::new_v4())
//             .event_time(OffsetDateTime::now_utc())
//             .strategy(Some(test_strategy_1()))
//             .instrument(test_inst_binance_btc_usdt_perp())
//             .side(MarketSide::Buy)
//             .quantity(dec!(1.0))
//             .price(dec!(50000.0))
//             .order_type(ExecutionOrderType::Maker)
//             .build();
//         let order_arc = Arc::new(order);

//         let mut rx = pubsub.subscriber().await;

//         order_manager.place_order(order_arc.clone()).await.unwrap();

//         // Verify that a VenueOrderUpdate event was published.
//         // We use a timeout in case no event is published.
//         let event = timeout(Duration::from_secs(1), rx.recv())
//             .await
//             .expect("Expected event within 1 sec")
//             .expect("Event channel closed");
//         match event {
//             Event::VenueOrderNew(new_order) => {
//                 assert_eq!(new_order.id, order_arc.id);
//                 assert_eq!(new_order.status, VenueOrderStatus::New);
//                 assert_eq!(new_order.side, MarketSide::Buy);
//                 assert_eq!(new_order.order_type, VenueOrderType::Limit);
//             }
//             _ => panic!("Expected VenueOrderNew event"),
//         }
//     }
// }
