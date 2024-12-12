#![allow(unused)]
use std::sync::Arc;

use arkin_binance::{
    prelude::{
        listen_key::NewListenKey,
        user_models::{BinancePositionSide, BinanceUSDMUserStreamEvent},
        BinanceSwapsListenKeyResponse,
    },
    BinanceHttpClient, BinanceWebSocketClient, CancelOpenOrders, NewOrder, Request,
};
use async_trait::async_trait;
use async_tungstenite::tungstenite::Message;
use dashmap::DashMap;
use futures_util::StreamExt;
use rust_decimal::Decimal;
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;

use crate::{Executor, ExecutorError};

#[derive(Debug, TypedBuilder)]
pub struct BinanceExecutor {
    pub pubsub: Arc<PubSub>,
    pub persistence: Arc<PersistenceService>,
    pub client: Arc<BinanceHttpClient>,
    pub api_key: String,
    #[builder(default)]
    pub open_orders: DashMap<Arc<Instrument>, Uuid>,
}

impl BinanceExecutor {
    pub async fn get_listen_key(&self) -> Result<String, ExecutorError> {
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
        Ok(listen_key.listen_key)
    }

    pub async fn handle_websocket_message(&self, msg: Message) -> Result<Option<Message>, ExecutorError> {
        debug!("Received message: {:?}", msg);

        let res = match msg {
            Message::Text(content) => {
                match serde_json::from_str::<BinanceUSDMUserStreamEvent>(&content) {
                    Ok(event) => self.handle_user_stream_update(event).await?,
                    Err(e) => {
                        error!("Error could not parse: {:?}", e);
                    }
                }
                None
            }
            Message::Binary(_vec) => {
                warn!("Received binary message");
                None
            }
            Message::Ping(vec) => {
                info!("Received ping");
                Some(Message::Pong(vec.into()))
            }
            Message::Pong(_vec) => {
                warn!("Received Pong");
                None
            }
            Message::Close(close_frame) => {
                error!("Received close frame: {:?}", close_frame);
                None
            }
            Message::Frame(frame) => {
                warn!("Received frame: {:?}", frame);
                None
            }
        };
        Ok(res)
    }

    pub async fn handle_user_stream_update(&self, event: BinanceUSDMUserStreamEvent) -> Result<(), ExecutorError> {
        debug!("Received user stream event: {:?}", event);
        match event {
            BinanceUSDMUserStreamEvent::OrderTradeUpdate {
                event_time,
                transaction_time,
                order,
            } => {
                if let Ok(instrument) = self.persistence.instrument_store.read_by_venue_symbol(&order.symbol).await {
                    // Check if there is a order.commission_asset
                    let commission_asset = if let Some(commission_asset) = order.commission_asset {
                        if let Ok(asset) = self.persistence.asset_store.read_by_symbol(&commission_asset).await {
                            Some(asset)
                        } else {
                            error!("Commission asset not found: {}", commission_asset);
                            None
                        }
                    } else {
                        None
                    };

                    let update = VenueOrderUpdate::builder()
                        .event_time(event_time)
                        .portfolio(test_portfolio())
                        .instrument(instrument)
                        .order_id(order.client_order_id)
                        .venue_order_id(order.order_id)
                        .side(order.side.into())
                        .order_type(order.order_type.into())
                        .time_in_force(order.time_in_force.into())
                        .price(order.original_price)
                        .quantity(order.original_quantity)
                        .fill_price(order.average_price)
                        .fill_quantity(order.filled_accumulated_quantity)
                        .last_fill_price(order.last_filled_price)
                        .last_fill_quantity(order.last_filled_quantity)
                        .commission_asset(commission_asset)
                        .commission(order.commission.unwrap_or(Decimal::ZERO))
                        .status(order.order_status.into())
                        .build()
                        .into();
                    self.pubsub.publish::<VenueOrderUpdate>(update);
                } else {
                    error!("Instrument not found: {}", order.symbol);
                }
            }
            BinanceUSDMUserStreamEvent::AccountUpdate {
                event_time,
                transaction_time,
                account,
            } => {
                for balance in &account.balances {
                    if let Ok(asset) = self.persistence.asset_store.read_by_symbol(&balance.asset).await {
                        let update = BalanceUpdate::builder()
                            .event_time(event_time)
                            .portfolio(test_portfolio())
                            .asset(asset)
                            .quantity(balance.wallet_balance)
                            .balance_change(balance.balance_change)
                            .build()
                            .into();
                        self.pubsub.publish::<BalanceUpdate>(update);
                    }
                }
                for position in &account.positions {
                    if let Ok(instrument) =
                        self.persistence.instrument_store.read_by_venue_symbol(&position.symbol).await
                    {
                        let position_side = match (position.position_side, position.position_amount) {
                            (BinancePositionSide::Long, _) => PositionSide::Long,
                            (BinancePositionSide::Short, _) => PositionSide::Short,
                            (BinancePositionSide::Both, q) => match q.is_sign_positive() {
                                true => PositionSide::Long,
                                false => PositionSide::Short,
                            },
                        };
                        let update = PositionUpdate::builder()
                            .event_time(event_time)
                            .portfolio(test_portfolio())
                            .instrument(instrument)
                            .entry_price(position.entry_price)
                            .quantity(position.position_amount)
                            .realized_pnl(position.accumulated_realized)
                            .unrealized_pnl(position.unrealized_pnl)
                            .position_side(position_side)
                            .build()
                            .into();
                        self.pubsub.publish::<PositionUpdate>(update);
                    }
                }
            }
            BinanceUSDMUserStreamEvent::MarginCall {
                event_time,
                cross_wallet_balance,
                positions,
            } => {
                // Handle the margin call
                unimplemented!("Margin call");
            }
            _ => {
                debug!("Unhandled event: {:?}", event);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl Executor for BinanceExecutor {
    async fn start(&self, shutdown: CancellationToken) -> Result<(), ExecutorError> {
        info!("Starting Binance executor...");

        let mut orders = self.pubsub.subscribe::<VenueOrder>();

        // Get listen key
        let mut listen_key_renewal_interval = tokio::time::interval(tokio::time::Duration::from_secs(1800));
        let listen_key = match self.get_listen_key().await {
            Ok(key) => key,
            Err(e) => {
                error!("Error: {:?}", e);
                return Err(ExecutorError::NetworkError(e.to_string()));
            }
        };

        let mut ws_client = match BinanceWebSocketClient::connect_with_listen_key(&listen_key).await {
            Ok((stream, _)) => {
                info!("Connected to Binance WebSocket");
                stream
            }
            Err(e) => {
                error!("Error: {:?}", e);
                return Err(ExecutorError::NetworkError(e.to_string()));
            }
        };

        loop {
            select! {
                _ = listen_key_renewal_interval.tick() => {
                    info!("Renewing listen key...");
                    let new_listen_key = match self.get_listen_key().await {
                        Ok(key) => key,
                        Err(e) => {
                            error!("Error: {:?}", e);
                            continue;
                        }
                    };
                    ws_client = match BinanceWebSocketClient::connect_with_listen_key(&new_listen_key).await {
                        Ok((stream, _)) => {
                            info!("Connected to Binance WebSocket");
                            stream
                        }
                        Err(e) => {
                            error!("Error: {:?}", e);
                            continue;
                        }
                    };
                }
                res = ws_client.as_mut().next() => {
                    debug!("Received message: {:?}", res);
                    match res {
                        Some(Ok(msg)) => {
                            match self.handle_websocket_message(msg).await {
                                Ok(Some(msg)) => {
                                    ws_client.socket.send(msg).await;
                                }
                                Ok(None) => {
                                    continue;
                                }
                                Err(e) => {
                                    error!("Error: {:?}", e);
                                }
                            }
                        }
                        Some(Err(e)) => {
                            error!("Error: {:?}", e);
                        }
                        None => {
                            error!("WebSocket stream closed");
                            // Reconnect
                            ws_client = match BinanceWebSocketClient::connect_with_listen_key(&listen_key).await {
                                Ok((stream, _)) => {
                                    info!("Connected to Binance WebSocket");
                                    stream
                                }
                                Err(e) => {
                                    error!("Error: {:?}", e);
                                    continue;
                                }
                            };
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

        if let Err(e) = self.client.send(req).await {
            error!("Error: {:?}", e);
            return Err(ExecutorError::NetworkError(e.to_string()));
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

        if let Err(e) = self.client.send(req).await {
            error!("Error: {:?}", e);
            return Err(ExecutorError::NetworkError(e.to_string()));
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
        let config = load::<PersistenceConfig>();
        let persistence = Arc::new(PersistenceService::from_config(&config, pubsub.clone()));

        let executor = Arc::new(
            BinanceExecutor::builder()
                .pubsub(pubsub.clone())
                .persistence(persistence.clone())
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

        let mut balance_updates = pubsub.subscribe::<BalanceUpdate>();
        let mut position_updates = pubsub.subscribe::<PositionUpdate>();
        let mut venue_order_updates = pubsub.subscribe::<VenueOrderUpdate>();
        let shutdown_clone = shutdown.clone();
        tracker.spawn(async move {
            loop {
                tokio::select! {
                    Ok(update) = balance_updates.recv() => {
                        info!("Received balance update: {}", update);
                    }
                    Ok(update) = position_updates.recv() => {
                        info!("Received position update: {}", update);
                    }
                    Ok(update) = venue_order_updates.recv() => {
                        info!("Received venue order update: {}", update);
                    }
                    _ = shutdown_clone.cancelled() => {
                        info!("Shutting down...");
                        break;
                    }
                }
            }
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
