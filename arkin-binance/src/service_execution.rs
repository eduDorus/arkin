use std::{env, pin::Pin, sync::Arc};

use async_trait::async_trait;
use reqwest::Client;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{
    common::{
        config::{ConfigurationRestApi, ConfigurationWebsocketApi},
        models::WebsocketMode,
        utils::SignatureGenerator,
    },
    derivatives_trading_usds_futures::{
        rest_api::RestApi,
        websocket_api::{
            CancelOrderParams, NewOrderParams, NewOrderPriceMatchEnum, NewOrderSideEnum, NewOrderTimeInForceEnum,
            StartUserDataStreamParams, WebsocketApi, WebsocketApiHandle,
        },
        DerivativesTradingUsdsFuturesRestApi, DerivativesTradingUsdsFuturesWsApi,
    },
};

const LISTEN_KEY_RENEWAL: u64 = 30 * 60;

#[derive(TypedBuilder)]
pub struct BinanceExecution {
    ws_handle: WebsocketApiHandle,
    ws_stream: RwLock<Option<WebsocketApi>>,
    http_handle: RestApi,
}

impl BinanceExecution {
    pub fn new() -> Arc<Self> {
        // Build Websocket trade api
        let api_key = env::var("API_KEY").expect("API_KEY must be set");
        let api_secret = env::var("API_SECRET").expect("API_SECRET must be set");
        let signature_gen = SignatureGenerator::new(Some(api_secret.clone()), None, None);
        let configuration = ConfigurationWebsocketApi::builder()
            .api_key(&api_key)
            .api_secret(&api_secret)
            .mode(WebsocketMode::Pool(3))
            .user_agent("unknown".to_owned())
            .signature_gen(signature_gen)
            .build();
        let handle = DerivativesTradingUsdsFuturesWsApi::production(configuration);

        let cfg = ConfigurationRestApi::builder()
            .api_key(&api_key)
            .api_secret(&api_secret)
            .client(Client::builder().gzip(true).build().expect("Failed to build HTTP client"))
            .user_agent("unknown".to_owned())
            .signature_gen(SignatureGenerator::new(Some(api_secret.clone()), None, None))
            .build();
        let rest_api = DerivativesTradingUsdsFuturesRestApi::production(cfg);

        Arc::new(Self {
            ws_handle: handle,
            ws_stream: RwLock::new(None),
            http_handle: rest_api,
        })
    }

    pub async fn place_order(&self, ctx: Arc<CoreCtx>, order: &VenueOrder) {
        info!(target: "execution::binance", "received new order {}", order.id);

        if let Some(api) = self.ws_stream.read().await.as_ref() {
            let side = match order.side {
                MarketSide::Buy => NewOrderSideEnum::Buy,
                MarketSide::Sell => NewOrderSideEnum::Sell,
            };
            let params = match order.order_type {
                VenueOrderType::Limit => match order.time_in_force {
                    VenueOrderTimeInForce::Gtc => NewOrderParams::builder()
                        .symbol(order.instrument.venue_symbol.clone())
                        .new_client_order_id(Some(order.id.to_string()))
                        .side(side)
                        .price(Some(order.price))
                        .quantity(Some(order.quantity))
                        .r#type("LIMIT".to_string())
                        .time_in_force(Some(NewOrderTimeInForceEnum::Gtc))
                        .build(),
                    VenueOrderTimeInForce::Gtx => NewOrderParams::builder()
                        .symbol(order.instrument.venue_symbol.clone())
                        .new_client_order_id(Some(order.id.to_string()))
                        .side(side)
                        .quantity(Some(order.quantity))
                        .r#type("LIMIT".to_string())
                        .time_in_force(Some(NewOrderTimeInForceEnum::Gtx))
                        .price_match(Some(NewOrderPriceMatchEnum::Queue))
                        .build(),
                    _ => {
                        error!(target: "execution::binance", "unsupported time in force type {}", order.time_in_force);
                        let mut order_clone = order.clone();
                        order_clone.reject(ctx.now().await);
                        ctx.publish(Event::VenueOrderRejected(order_clone.into())).await;
                        return;
                    }
                },
                VenueOrderType::Market => NewOrderParams::builder()
                    .symbol(order.instrument.venue_symbol.clone())
                    .new_client_order_id(Some(order.id.to_string()))
                    .side(side)
                    .r#type("MARKET".to_string())
                    .quantity(Some(order.quantity))
                    // .price(Some(order.price))
                    // .time_in_force(Some(NewOrderTimeInForceEnum::Gtx))
                    .build(),
                _ => {
                    error!(target: "execution::binance", "unsupported order type {}", order.order_type);
                    let mut order_clone = order.clone();
                    order_clone.reject(ctx.now().await);
                    ctx.publish(Event::VenueOrderRejected(order_clone.into())).await;
                    return;
                }
            };

            match api.new_order(params).await {
                Ok(res) => {
                    let data = match res.data() {
                        Ok(d) => d,
                        Err(e) => {
                            error!(target: "execution::binance", "data error: {}", e);
                            let mut order_clone = order.clone();
                            order_clone.reject(ctx.now().await);
                            ctx.publish(Event::VenueOrderRejected(order_clone.into())).await;
                            return;
                        }
                    };
                    debug!(target: "execution::binance", "place order response data: {:?}", data);
                    let mut order_clone = order.clone();
                    order_clone.place(ctx.now().await);
                    ctx.publish(Event::VenueOrderPlaced(order_clone.into())).await;
                }
                Err(e) => {
                    error!(target: "execution::binance", "place error: {}", e);
                    let mut order_clone = order.clone();
                    order_clone.reject(ctx.now().await);
                    ctx.publish(Event::VenueOrderRejected(order_clone.into())).await;
                }
            }
        } else {
            error!(target: "execution::binance", "API not connected");
            let mut order_clone = order.clone();
            order_clone.reject(ctx.now().await);
            ctx.publish(Event::VenueOrderRejected(order_clone.into())).await;
            return;
        }
    }

    pub async fn cancel_order(&self, ctx: Arc<CoreCtx>, order: &VenueOrder) {
        info!(target: "execution::binance", "received cancel order for {}", order.id);

        if let Some(api) = self.ws_stream.read().await.as_ref() {
            let params = CancelOrderParams::builder()
                .symbol(order.instrument.venue_symbol.clone())
                .orig_client_order_id(order.id.to_string())
                .build();

            match api.cancel_order(params).await {
                Ok(res) => {
                    let data = match res.data() {
                        Ok(d) => d,
                        Err(e) => {
                            error!(target: "execution::binance", "data error: {}", e);
                            return;
                        }
                    };
                    info!(target: "execution::binance", "cancel order response data: {:?}", data);
                    let mut order_clone = order.clone();
                    order_clone.cancel(ctx.now().await);
                    ctx.publish(Event::VenueOrderCancelled(order_clone.into())).await;
                }
                Err(e) => {
                    error!(target: "execution::binance", "cancel error: {}", e);
                }
            }
        } else {
            error!(target: "execution::binance", "API not connected");
            return;
        };
    }
}

async fn binance_exec_task(exec: Arc<BinanceExecution>, _ctx: Arc<CoreCtx>, service_ctx: Arc<ServiceCtx>) {
    let mut rx = if let Some(api) = exec.ws_stream.read().await.as_ref() {
        api.subscribe_on_ws_message()
    } else {
        error!(target: "execution::binance", "WebSocket API not connected, could not start task");
        return;
    };

    // Create interval every 30mins to renew listen key
    let mut listen_key_renewal = tokio::time::interval(std::time::Duration::from_secs(LISTEN_KEY_RENEWAL));

    let shutdown = service_ctx.get_shutdown_token();
    loop {
        tokio::select! {
          Ok(msg) = rx.recv() => {
              info!(target: "execution::binance", "received WebSocket message: {:?}", msg);
          }
          _ = listen_key_renewal.tick() => {
              info!(target: "execution::binance", "renewing listen key");
              if let Err(e) = exec.http_handle.keepalive_user_data_stream().await {
                  error!(target: "execution::binance", "Failed to renew listen key: {}", e);
              }
          }
          _ = shutdown.cancelled() => {
              info!(target: "execution::binance", "shutting down");
              break;
          }
        }
    }
}

#[async_trait]
impl Runnable for BinanceExecution {
    async fn handle_event(&self, ctx: Arc<CoreCtx>, event: Event) {
        match &event {
            Event::NewVenueOrder(o) => self.place_order(ctx, o).await,
            Event::CancelVenueOrder(o) => self.cancel_order(ctx, o).await,
            e => warn!(target: "execution::binance", "received unused event {}", e),
        }
    }

    async fn setup(&self, _ctx: Arc<ServiceCtx>, _core_ctx: Arc<CoreCtx>) {
        // Connect ws api
        match self.ws_handle.connect().await {
            Ok(api) => {
                let params = StartUserDataStreamParams::builder().build();
                let res = api
                    .start_user_data_stream(params)
                    .await
                    .expect("Failed to start user data stream")
                    .data()
                    .expect("Failed to get data from user data stream");
                info!(target: "execution::binance", "start_user_data_stream data: {:?}", res);
                let mut streams_guard = self.ws_stream.write().await;
                *streams_guard = Some(api);
            }
            Err(e) => error!(target: "execution::binance", "Connect to binance failed: {}", e),
        }
    }

    async fn get_tasks(
        self: Arc<Self>,
        service_ctx: Arc<ServiceCtx>,
        core_ctx: Arc<CoreCtx>,
    ) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
        vec![Box::pin(binance_exec_task(self, core_ctx.clone(), service_ctx.clone()))]
    }

    async fn teardown(&self, _ctx: Arc<ServiceCtx>, _core_ctx: Arc<CoreCtx>) {
        // Disconnect trade stream
        if let Some(api) = self.ws_stream.read().await.as_ref() {
            if let Err(e) = api.disconnect().await {
                error!(target: "execution::binance", "Disconnect error from stream api: {}", e);
            } else {
                info!(target: "execution::binance", "Disconnected successfully");
            }
        }

        // Delete listen key
        if let Err(e) = self.http_handle.close_user_data_stream().await {
            error!(target: "execution::binance", "Failed to delete listen key: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{env, time::Duration};

    use anyhow::{Context, Result};
    use reqwest::Client;
    use rust_decimal::prelude::*;
    use tracing::info;
    use uuid::Uuid;

    use crate::{
        common::{
            config::{ConfigurationRestApi, ConfigurationWebsocketApi, ConfigurationWebsocketStreams},
            models::WebsocketMode,
            utils::SignatureGenerator,
        },
        derivatives_trading_usds_futures::{
            rest_api::CurrentAllOpenOrdersParams,
            websocket_api::{
                AccountInformationV2Params, CancelOrderParams, FuturesAccountBalanceV2Params,
                KeepaliveUserDataStreamParams, NewOrderParams, NewOrderSideEnum, NewOrderTimeInForceEnum,
                PositionInformationV2Params, StartUserDataStreamParams,
            },
            websocket_streams::{AggregateTradeStreamsParams, IndividualSymbolBookTickerStreamsParams},
            DerivativesTradingUsdsFuturesRestApi, DerivativesTradingUsdsFuturesWsApi,
            DerivativesTradingUsdsFuturesWsStreams,
        },
    };

    #[tokio::test]
    #[test_log::test]
    #[ignore = "Long running test, only run manually"]
    async fn subscribe_user_stream() -> Result<()> {
        // Load credentials from env
        let api_key = env::var("API_KEY").expect("API_KEY must be set in the environment");
        let api_secret = env::var("API_SECRET").expect("API_SECRET must be set in the environment");

        let cfg = ConfigurationRestApi::builder()
            .api_key(&api_key)
            .api_secret(&api_secret)
            .client(Client::builder().gzip(true).build().expect("Failed to build HTTP client"))
            .user_agent("unknown".to_owned())
            .signature_gen(SignatureGenerator::new(Some(api_secret.clone()), None, None))
            .build();
        let rest_api = DerivativesTradingUsdsFuturesRestApi::production(cfg);

        // Build WebSocket API config
        let signature_gen = SignatureGenerator::new(Some(api_secret.clone()), None, None);
        let configuration = ConfigurationWebsocketApi::builder()
            .api_key(&api_key)
            .api_secret(&api_secret)
            .mode(WebsocketMode::Pool(3)) // Use pool mode with a pool size of 3
            .user_agent("unknown".to_owned())
            .signature_gen(signature_gen)
            .build();

        let client = DerivativesTradingUsdsFuturesWsApi::production(configuration);
        let connection = client.connect().await?;

        // Subscribe to the stream
        // let _stream = connection
        //     .subscribe_on_ws_events(|e| info!(target: "execution::binance", "USER DATA SUBSCRIPTION STREAM: {:?}", e));
        let mut rx = connection.subscribe_on_ws_message();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                info!("SUBSCRIPTION: {:?}", event);
            }
        });

        let params = StartUserDataStreamParams::builder().build();
        let res = connection.start_user_data_stream(params).await?.data()?;
        info!(target: "execution::binance", "start_user_data_stream data: {:?}", res);

        // let params = AccountInformationV2Params::builder().build();
        // let response = connection.account_information_v2(params).await?;
        // let data = response.data()?;
        // info!(target: "execution::binance", ?data, "account information data");

        let client_order_id = Uuid::new_v4();
        let params = NewOrderParams::builder()
            .side(NewOrderSideEnum::Buy)
            .price(Some(dec!(110000)))
            .quantity(Some(dec!(0.001)))
            .r#type("LIMIT")
            .symbol("btcusdt")
            .time_in_force(Some(NewOrderTimeInForceEnum::Gtc))
            .new_client_order_id(Some(client_order_id.to_string()))
            .build();
        let res = connection.new_order(params).await?.data()?;
        info!(target: "execution::binance", "new order data: {:?}", res);

        tokio::time::sleep(Duration::from_secs(3)).await;
        let res = rest_api
            .current_all_open_orders(CurrentAllOpenOrdersParams::builder().build())
            .await
            .context("Failed to get current all open orders")?
            .data()
            .await?;
        info!(target: "execution::binance", "current all open orders data: {:?}", res);
        info!(target: "execution::binance", "current all open orders count: {}", res.len());

        let params = CancelOrderParams::builder()
            .symbol("btcusdt")
            .orig_client_order_id(client_order_id.to_string())
            .build();
        let res = connection.cancel_order(params).await?.data()?;
        info!(target: "execution::binance", "cancel order data: {:?}", res);

        connection
            .keepalive_user_data_stream(KeepaliveUserDataStreamParams::builder().build())
            .await?;

        let res = connection
            .account_information_v2(AccountInformationV2Params::builder().build())
            .await?
            .data()?;
        info!(target: "execution::binance", "account information data: {:?}", res);

        let res = connection
            .futures_account_balance_v2(FuturesAccountBalanceV2Params::builder().build())
            .await?
            .data()?;
        info!(target: "execution::binance", "futures account balance data: {:?}", res);

        let res = connection
            .position_information_v2(PositionInformationV2Params::builder().build())
            .await?
            .data()?;
        info!(target: "execution::binance", "position information data: {:?}", res);

        // Disconnect after 10 seconds
        tokio::time::sleep(Duration::from_secs(10)).await;
        connection.disconnect().await.context("Failed to disconnect WebSocket client")?;

        Ok(())
    }

    #[tokio::test]
    #[test_log::test]
    #[ignore = "Long running test, only run manually"]
    async fn subscribe_binance_agg_trades() -> Result<()> {
        // Build WebSocket Streams config
        let ws_streams_conf = ConfigurationWebsocketStreams::builder().mode(WebsocketMode::Pool(3)).build();

        // Create the DerivativesTradingUsdsFutures WebSocket Streams client
        let ws_streams_client = DerivativesTradingUsdsFuturesWsStreams::production(ws_streams_conf);

        // Connect to WebSocket
        let connection = ws_streams_client
            .connect()
            .await
            .context("Failed to connect to WebSocket Streams")?;

        // Subscribe to the streams
        connection
            .individual_symbol_book_ticker_streams(
                IndividualSymbolBookTickerStreamsParams::builder()
                    .symbol("btcusdt".to_string())
                    .build(),
            )
            .await
            .context("Failed to subscribe to the stream")?;

        // Subscribe to the streams
        connection
            .individual_symbol_book_ticker_streams(
                IndividualSymbolBookTickerStreamsParams::builder()
                    .symbol("ethusdt".to_string())
                    .build(),
            )
            .await
            .context("Failed to subscribe to the stream")?;

        connection
            .individual_symbol_book_ticker_streams(
                IndividualSymbolBookTickerStreamsParams::builder()
                    .symbol("solusdt".to_string())
                    .build(),
            )
            .await
            .context("Failed to subscribe to the stream")?;

        connection
            .aggregate_trade_streams(AggregateTradeStreamsParams::builder().symbol("btcusdt".to_string()).build())
            .await
            .context("Failed to subscribe to the stream")?;
        connection
            .aggregate_trade_streams(AggregateTradeStreamsParams::builder().symbol("ethusdt".to_string()).build())
            .await
            .context("Failed to subscribe to the stream")?;
        connection
            .aggregate_trade_streams(AggregateTradeStreamsParams::builder().symbol("solusdt".to_string()).build())
            .await
            .context("Failed to subscribe to the stream")?;

        let mut rx = connection.subscribe_on_ws_message();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                info!("{:?}", event);
            }
        });

        // Disconnect after 20 seconds
        tokio::time::sleep(Duration::from_secs(20)).await;
        connection.disconnect().await.context("Failed to disconnect WebSocket client")?;

        Ok(())
    }
}
