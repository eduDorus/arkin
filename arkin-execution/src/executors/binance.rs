use std::sync::Arc;

use arkin_binance::{
    prelude::{listen_key::NewListenKey, user_models::BinanceUSDMUserStreamEvent, BinanceSwapsListenKeyResponse},
    BinanceHttpClient, BinanceWebSocketClient, CancelOpenOrders, NewOrder, Request,
};
use arkin_core::{Instrument, PubSub, VenueOrder, VenueOrderId, VenueOrderType};
use async_trait::async_trait;
use async_tungstenite::tungstenite::Message;
use dashmap::DashMap;
use futures_util::StreamExt;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use crate::{Executor, ExecutorError};

#[derive(Debug, TypedBuilder)]
pub struct BinanceExecutor {
    pub pubsub: Arc<PubSub>,
    pub client: Arc<BinanceHttpClient>,
    pub api_key: String,
    #[builder(default)]
    pub open_orders: DashMap<Arc<Instrument>, Uuid>,
}

#[async_trait]
impl Executor for BinanceExecutor {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), ExecutorError> {
        info!("Starting Binance executor...");

        let mut orders = self.pubsub.subscribe::<VenueOrder>();

        let req: Request = NewListenKey::new().into();
        let listen_key = match self.client.send(req).await {
            Ok(res) => {
                // deserialize json response
                info!("Response: {:?}", res.body);
                match serde_json::from_str::<BinanceSwapsListenKeyResponse>(&res.body) {
                    Ok(res) => res,
                    Err(e) => {
                        error!("Error: {:?}", e);
                        return Err(ExecutorError::NetworkError(e.to_string()));
                    }
                }
            }
            Err(e) => {
                error!("Error: {:?}", e);
                return Err(ExecutorError::NetworkError(e.to_string()));
            }
        };

        let mut ws_client = match BinanceWebSocketClient::connect_with_listen_key(&listen_key.listen_key).await {
            Ok((stream, _)) => {
                info!("Connected to Binance WebSocket");
                stream
            }
            Err(e) => {
                error!("Error: {:?}", e);
                return Err(ExecutorError::NetworkError(e.to_string()));
            }
        };

        // let stream: Stream = UserDataStream::new().into();
        // ws_client.subscribe(vec![&stream]).await;
        // info!("Subscribed to user data stream");

        loop {
            select! {
                Some(res) = ws_client.as_mut().next() => {
                    debug!("Received message: {:?}", res);
                    let msg = match res {
                        Ok(msg) => msg,
                        Err(e) => {
                            error!("Error: {:?}", e);
                            continue;
                        }
                    };

                    if msg.is_ping() {
                        info!("Received ping");
                        let new_msg = Message::Pong(msg.into_data());
                        // Send pong
                        if let Err(e) = ws_client.socket.send(new_msg).await {
                            error!("Error sending Pong: {:?}", e);
                        }
                        continue;
                    }

                    let msg_content = match  msg.into_text() {
                      Ok(content) => content,
                      Err(e) => {
                        error!("Error: {:?}", e);
                        continue;
                      }
                    };

                    // Let's serialize this stuff
                    match serde_json::from_str::<BinanceUSDMUserStreamEvent>(&msg_content) {
                        Ok(event) => {
                            info!("Event: {:?}", event);
                        }
                        Err(e) => {
                            error!("Error: {:?}", e);
                        }
                    }
                }
                Ok(order) = orders.recv() => {
                    info!("BinanceExecutor received order: {}", order.instrument);

                    // First cancel all open orders for the instrument
                    if let Err(e) = self.cancel_orders_by_instrument(order.instrument.clone()).await {
                        error!("Failed to cancel open orders: {}", e);
                    }

                    if let Err(e) = self.place_order(order.clone()).await {
                        error!("Failed to process order: {}", e);
                    }
                }
                _ = shutdown.cancelled() => {
                    info!("Shutting down Binance executor...");
                    info!("Cancelling all open orders");
                    let instruments = self.open_orders.iter().map(|e| e.key().clone()).collect::<Vec<_>>();
                    for inst in instruments {
                      self.cancel_orders_by_instrument(inst).await?;
                    }
                    break;
                }
            }
        }
        Ok(())
    }

    async fn place_order(&self, order: Arc<VenueOrder>) -> Result<(), ExecutorError> {
        self.open_orders.insert(order.instrument.clone(), order.id);

        let req: Request = NewOrder::builder()
            .symbol(order.instrument.venue_symbol.clone())
            .order_type(order.order_type.into())
            .side(order.side.into())
            .price(order.price)
            .quantity(order.quantity.into())
            .time_in_force(if order.order_type == VenueOrderType::Market {
                None
            } else {
                Some(order.time_in_force.into())
            })
            .new_client_order_id(order.id.to_string().into())
            .build()
            .into();

        let res = self.client.send(req).await;
        match res {
            Ok(res) => {
                info!("Response: {:?}", res);
            }
            Err(e) => {
                error!("Error: {:?}", e);
            }
        }
        Ok(())
    }
    async fn place_orders(&self, _orders: Vec<Arc<VenueOrder>>) -> Result<(), ExecutorError> {
        unimplemented!()
    }

    async fn modify_order(&self, _order: Arc<VenueOrder>) -> Result<(), ExecutorError> {
        unimplemented!()
    }
    async fn modify_orders(&self, _order: Vec<Arc<VenueOrder>>) -> Result<(), ExecutorError> {
        unimplemented!()
    }

    async fn cancel_order(&self, _id: VenueOrderId) -> Result<(), ExecutorError> {
        unimplemented!()
    }
    async fn cancel_orders(&self, _ids: Vec<VenueOrderId>) -> Result<(), ExecutorError> {
        unimplemented!()
    }

    async fn cancel_orders_by_instrument(&self, instrument: Arc<Instrument>) -> Result<(), ExecutorError> {
        let req: Request = CancelOpenOrders::builder()
            .symbol(instrument.venue_symbol.clone())
            .build()
            .into();

        let res = self.client.send(req).await;
        match res {
            Ok(res) => {
                info!("Response: {:?}", res);
            }
            Err(e) => {
                error!("Error: {:?}", e);
            }
        }
        Ok(())
    }
    async fn cancel_all_orders(&self) -> Result<(), ExecutorError> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    use arkin_binance::Credentials;
    use arkin_core::prelude::*;
    use rust_decimal_macros::dec;
    use test_log::test;
    use tokio_rustls::rustls::crypto::{aws_lc_rs, CryptoProvider};
    use tokio_util::task::TaskTracker;

    #[test(tokio::test)]
    async fn test_binance_executor() {
        CryptoProvider::install_default(aws_lc_rs::default_provider())
            .expect("Failed to install default CryptoProvider");

        // Create executor
        let pubsub = Arc::new(PubSub::new());
        let executor = Arc::new(
            BinanceExecutor::builder()
                .pubsub(pubsub.clone())
                .client(Arc::new(
                    BinanceHttpClient::builder()
                        .credentials(Some(Credentials::from_hmac(
                            "ppCYOYKlKLRVwGCzmcbXNf2Qn34aeDEN36A4I0Fwdj8WmpvfkxO9cmNIx5PwhmOd",
                            "cs4wa0w860lgkViblUzua4ThRXpfD22ruG8d0GytU7fIrJCvz8jvCAzKpaKPwTl0",
                        )))
                        .build(),
                ))
                .api_key("ppCYOYKlKLRVwGCzmcbXNf2Qn34aeDEN36A4I0Fwdj8WmpvfkxO9cmNIx5PwhmOd".to_string())
                .build(),
        );

        // Start executor
        let tracker = TaskTracker::new();
        let shutdown = CancellationToken::new();
        let shutdown_clone = shutdown.clone();
        tracker.spawn(async move {
            executor.start(shutdown_clone).await.unwrap();
        });

        // Wait for executor to start
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        // Create a sample VenueOrder
        let order: Arc<VenueOrder> = VenueOrder::builder()
            .id(Uuid::new_v4())
            .portfolio(test_portfolio())
            .instrument(test_inst_binance_eth_usdt_perp())
            .order_type(VenueOrderType::Limit)
            .side(MarketSide::Buy)
            .price(Some(dec!(3800.00)))
            .quantity(dec!(0.01))
            .build()
            .into();

        info!("Publishing Venue Order");
        pubsub.publish::<VenueOrder>(order);

        // // Subscribe to fill and updates
        // let mut updates = pubsub.subscribe::<VenueOrderState>();
        // let mut fills = pubsub.subscribe::<VenueOrderFill>();

        // // Publish the order
        // info!("Publishing order: {:?}", order);
        // pubsub.publish::<VenueOrder>(order.clone());

        // // Check for ack
        // let ack = updates.recv().await.unwrap();
        // assert_eq!(ack.status, VenueOrderStatus::Placed);

        // // Send price update
        // let tick = Arc::new(
        //     Tick::builder()
        //         .instrument(test_inst_binance_btc_usdt_perp())
        //         .tick_id(0 as u64)
        //         .bid_price(dec!(50000))
        //         .bid_quantity(dec!(1))
        //         .ask_price(dec!(50001))
        //         .ask_quantity(dec!(1))
        //         .build(),
        // );
        // pubsub.publish::<Tick>(tick);

        // // Check for fill
        // let fill = fills.recv().await.unwrap();
        // assert_eq!(fill.price, Decimal::from_f64(50001.).unwrap());
        // assert_eq!(fill.quantity, Decimal::from_f64(0.1).unwrap());

        tokio::time::sleep(Duration::from_secs(10)).await;

        let market_order: Arc<VenueOrder> = VenueOrder::builder()
            .id(Uuid::new_v4())
            .portfolio(test_portfolio())
            .instrument(test_inst_binance_eth_usdt_perp())
            .order_type(VenueOrderType::Market)
            .side(MarketSide::Buy)
            .price(None)
            .quantity(dec!(0.006))
            .build()
            .into();
        pubsub.publish::<VenueOrder>(market_order);

        tokio::time::sleep(Duration::from_secs(10)).await;

        match tokio::signal::ctrl_c().await {
            Ok(_) => {
                info!("Received Ctrl-C signal, shutting down...");
            }
            Err(e) => error!("Failed to listen for Ctrl-C signal: {}", e),
        }

        // Shutdown
        shutdown.cancel();
        tracker.close();
        tracker.wait().await;
    }
}
