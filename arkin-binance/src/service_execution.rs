use std::{collections::HashMap, env, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{
    common::{
        config::{ConfigurationWebsocketApi, ConfigurationWebsocketStreams},
        models::WebsocketMode,
        utils::SignatureGenerator,
    },
    derivatives_trading_usds_futures::{
        websocket_api::{
            CancelOrderParams, NewOrderParams, NewOrderPriceMatchEnum, NewOrderSideEnum, NewOrderTimeInForceEnum,
            StartUserDataStreamParams, SymbolOrderBookTickerParams, WebsocketApi, WebsocketApiHandle,
        },
        websocket_streams::{
            AggregateTradeStreamsParams, IndividualSymbolBookTickerStreamsParams, WebsocketStreams,
            WebsocketStreamsHandle,
        },
        DerivativesTradingUsdsFuturesWsApi, DerivativesTradingUsdsFuturesWsStreams,
    },
};

#[derive(TypedBuilder)]
pub struct BinanceExecution {
    identifier: String,
    time: Arc<dyn SystemTime>,
    instruments: Vec<Arc<Instrument>>,
    publisher: Arc<dyn Publisher>,
    handle: WebsocketApiHandle,
    api: Arc<RwLock<Option<WebsocketApi>>>,
    stream_handle: WebsocketStreamsHandle,
    stream_api: Arc<RwLock<Option<WebsocketStreams>>>,
}

impl BinanceExecution {
    pub fn new(
        time: Arc<dyn SystemTime>,
        publisher: Arc<dyn Publisher>,
        instruments: Vec<Arc<Instrument>>,
    ) -> Arc<Self> {
        // Build Websocket trade api
        let api_key = env::var("API_KEY").expect("API_KEY must be set");
        let api_secret = env::var("API_SECRET").expect("API_SECRET must be set");
        let signature_gen = SignatureGenerator::new(Some(api_secret.clone()), None, None);
        let configuration = ConfigurationWebsocketApi::builder()
            .api_key(api_key)
            .api_secret(api_secret)
            .mode(WebsocketMode::Pool(1))
            .user_agent("unknown".to_owned())
            .signature_gen(signature_gen)
            .build();
        let handle = DerivativesTradingUsdsFuturesWsApi::production(configuration);

        // Build WebSocket Stream
        let stream_configuration = ConfigurationWebsocketStreams::builder().build();
        let stream_handle = DerivativesTradingUsdsFuturesWsStreams::production(stream_configuration);

        Arc::new(Self {
            identifier: "execution::binance".to_owned(),
            time,
            instruments,
            publisher,
            handle,
            api: Arc::new(RwLock::new(None)),
            stream_handle,
            stream_api: Arc::new(RwLock::new(None)),
        })
    }

    #[instrument(skip_all, fields(service = %self.identifier))]
    pub async fn place_order(&self, order: &VenueOrder) {
        info!(target: "execution::binance", "received new order {}", order.id);
        let api_guard = self.api.read().await;
        let Some(api) = api_guard.as_ref() else {
            error!(target: "execution::binance", "API not connected");
            let mut order_clone = order.clone();
            order_clone.reject(self.time.now().await);
            self.publisher.publish(Event::VenueOrderRejected(order_clone.into())).await;
            return;
        };

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
                    order_clone.reject(self.time.now().await);
                    self.publisher.publish(Event::VenueOrderRejected(order_clone.into())).await;
                    return;
                }
            },
            VenueOrderType::Market => NewOrderParams::builder()
                .symbol(order.instrument.venue_symbol.clone())
                .new_client_order_id(Some(order.id.to_string()))
                .side(side)
                .price(Some(order.price))
                .quantity(Some(order.quantity))
                .r#type("MARKET".to_string())
                .time_in_force(Some(NewOrderTimeInForceEnum::Gtx))
                .build(),
            _ => {
                error!(target: "execution::binance", "unsupported order type {}", order.order_type);
                let mut order_clone = order.clone();
                order_clone.reject(self.time.now().await);
                self.publisher.publish(Event::VenueOrderRejected(order_clone.into())).await;
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
                        order_clone.reject(self.time.now().await);
                        self.publisher.publish(Event::VenueOrderRejected(order_clone.into())).await;
                        return;
                    }
                };
                debug!(target: "execution::binance", "place order response data: {:?}", data);
                let mut order_clone = order.clone();
                order_clone.place(self.time.now().await);
                self.publisher.publish(Event::VenueOrderPlaced(order_clone.into())).await;
            }
            Err(e) => {
                error!(target: "execution::binance", "place error: {}", e);
                let mut order_clone = order.clone();
                order_clone.reject(self.time.now().await);
                self.publisher.publish(Event::VenueOrderRejected(order_clone.into())).await;
            }
        }
    }

    #[instrument(skip_all, fields(service = %self.identifier))]
    pub async fn cancel_order(&self, order: &VenueOrder) {
        info!(target: "execution::binance", "received cancel order for {}", order.id);
        let api_guard = self.api.read().await;
        let Some(api) = api_guard.as_ref() else {
            error!(target: "execution::binance", "API not connected");
            return;
        };

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
                debug!(target: "execution::binance", "cancel order response data: {:?}", data);
                let mut order_clone = order.clone();
                order_clone.cancel(self.time.now().await);
                self.publisher.publish(Event::VenueOrderCancelled(order_clone.into())).await;
            }
            Err(e) => {
                error!(target: "execution::binance", "cancel error: {}", e);
            }
        }
    }

    #[instrument(skip_all, fields(service = %self.identifier))]
    pub async fn tick_update(&self, tick: &Tick) {
        info!(target: "execution::binance", "received tick {}", tick);
    }
}

#[async_trait]
impl Runnable for BinanceExecution {
    async fn handle_event(&self, _core_ctx: Arc<CoreCtx>, event: Event) {
        match &event {
            Event::NewVenueOrder(o) => self.place_order(o).await,
            Event::CancelVenueOrder(o) => self.cancel_order(o).await,
            Event::TickUpdate(t) => self.tick_update(t).await,
            e => warn!(target: "execution::binance", "received unused event {}", e),
        }
    }

    async fn setup(&self, _ctx: Arc<ServiceCtx>, _core_ctx: Arc<CoreCtx>) {
        // Connect market stream
        let mut streams_guard = self.stream_api.write().await;
        match self.stream_handle.connect().await {
            Ok(api) => {
                let inst_lookup = self
                    .instruments
                    .iter()
                    .map(|i| (i.venue_symbol.clone(), i.clone()))
                    .collect::<HashMap<_, _>>();
                for inst in &self.instruments {
                    // Setup the stream parameters
                    let params = IndividualSymbolBookTickerStreamsParams::builder()
                        .symbol(inst.venue_symbol.to_owned())
                        .build();

                    // Subscribe to the stream
                    let stream = api
                        .individual_symbol_book_ticker_streams(params)
                        .await
                        .expect("Failed to subscribe to the stream");

                    // Register callback for incoming messages
                    let publisher = self.publisher.clone();
                    let inst_lookup_ticks = inst_lookup.clone();
                    stream.on_message(move |data| {
                        if let Some(inst) = inst_lookup_ticks.get(&data.instrument) {
                            let tick = Tick::builder()
                                .instrument(inst.clone())
                                .tick_id(data.update_id)
                                .event_time(data.event_time)
                                .ask_price(data.ask_price)
                                .ask_quantity(data.ask_quantity)
                                .bid_price(data.bid_price)
                                .bid_quantity(data.bid_quantity)
                                .build();
                            let publisher = publisher.clone();
                            tokio::spawn(async move { publisher.publish(Event::TickUpdate(tick.into())).await });
                        } else {
                            warn!(target: "execution::binance", "could not find instrument: {}", data.instrument)
                        }
                    });

                    // Setup the stream parameters
                    let params = AggregateTradeStreamsParams::builder()
                        .symbol(inst.venue_symbol.to_owned())
                        .build();

                    // Subscribe to the stream
                    let stream = api
                        .aggregate_trade_streams(params)
                        .await
                        .expect("Failed to subscribe to the stream");

                    // Register callback for incoming messages
                    let publisher = self.publisher.clone();
                    let inst_lookup_agg_trades = inst_lookup.clone();
                    stream.on_message(move |data| {
                        if let Some(inst) = inst_lookup_agg_trades.get(&data.instrument) {
                            let side = if data.maker {
                                MarketSide::Sell
                            } else {
                                MarketSide::Buy
                            };
                            let trade = AggTrade::builder()
                                .instrument(inst.clone())
                                .trade_id(data.agg_trade_id)
                                .event_time(data.event_time)
                                .side(side)
                                .price(data.price)
                                .quantity(data.quantity)
                                .build();
                            let publisher = publisher.clone();
                            tokio::spawn(async move { publisher.publish(Event::AggTradeUpdate(trade.into())).await });
                        } else {
                            error!(target: "execution::binance", "could not find instrument: {}", data.instrument)
                        }
                    });
                }

                *streams_guard = Some(api);
            }
            Err(e) => error!(target: "execution::binance", "Connect to binance streams failed: {}", e),
        }

        // Connect trade api
        let mut api_guard = self.api.write().await;
        match self.handle.connect().await {
            Ok(api) => {
                info!(target: "execution::binance", "Connected successfully");

                // Subscribe to user data stream
                let params = StartUserDataStreamParams::builder().build();
                let response = api
                    .start_user_data_stream(params)
                    .await
                    .expect("Could not start user data stream");
                debug!(target: "execution::binance", "start_user_data_stream rate limits: {:?}", response.rate_limits);
                let data = response.data().expect("Subscription error");
                debug!(target: "execution::binance", "start_user_data_stream data: {:?}", data);

                // Subscribe to symbol price ticker
                let params = SymbolOrderBookTickerParams::builder().symbol("btcusdt".to_owned()).build();
                let response = api
                    .symbol_order_book_ticker(params)
                    .await
                    .expect("Could not start order book stream");
                info!(target: "execution::binance", "symbol_order_book_ticker rate limits: {:?}", response.rate_limits);
                let data = response.data().expect("Subscription error");
                info!(target: "execution::binance", "symbol_order_book_ticker data: {:?}", data);

                // Get a subscription handle
                // let _stream =
                //     api.subscribe_on_ws_events(|e| debug!(target: "execution::binance", "USER STREAM EVENT: {:?}", e));

                *api_guard = Some(api);
            }
            Err(e) => error!(target: "execution::binance", "Connect to binance trade api failed: {}", e),
        }
    }

    #[instrument(skip_all, fields(service = %self.identifier))]
    async fn teardown(&self, _ctx: Arc<ServiceCtx>, _core_ctx: Arc<CoreCtx>) {
        // Disconnect trade stream
        let api_guard = self.stream_api.read().await;
        if let Some(api) = api_guard.as_ref() {
            if let Err(e) = api.disconnect().await {
                error!(target: "execution::binance", "Disconnect error from stream api: {}", e);
            } else {
                info!(target: "execution::binance", "Disconnected successfully");
            }
        }

        // Disconnect trade api
        let api_guard = self.api.read().await;
        if let Some(api) = api_guard.as_ref() {
            if let Err(e) = api.disconnect().await {
                error!(target: "execution::binance", "Disconnect error from trade api: {}", e);
            } else {
                info!(target: "execution::binance", "Disconnected successfully");
            }
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
            config::{AgentConnector, ConfigurationRestApi, ConfigurationWebsocketApi, ConfigurationWebsocketStreams},
            models::WebsocketMode,
            utils::SignatureGenerator,
            websocket::WebsocketHandler,
        },
        derivatives_trading_usds_futures::{
            rest_api::{CurrentAllOpenOrdersParams, RestApi},
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
