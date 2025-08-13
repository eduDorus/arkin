use std::{env, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use tracing::{debug, error, info, warn};

use arkin_core::prelude::*;
use url::Url;

use crate::{
    http::{Credentials, HttpClient},
    usdm::{
        models::user_models::{BinanceOrderType, BinanceTimeInForce},
        trade::{BinanceOrder, CancelOrderRequest, NewOrderRequest},
    },
    ws::{response::EmptyResult, MarketWsManager, UserWsManager, WsEvent, WsManager},
};

#[allow(dead_code)]
pub struct BinanceExecutor {
    venue: Arc<Venue>,
    ws_user: WsManager,
    ws_market: WsManager,
    http_client: HttpClient,
}

impl BinanceExecutor {
    pub fn new(venue: Arc<Venue>, ws_manager: WsManager, credentials: Credentials) -> Arc<Self> {
         let api_key = env::var("API_KEY").expect("API_KEY must be set");
    let api_secret = env::var("API_SECRET").expect("API_SECRET must be set");
        let http_client = HttpClient::builder().credentials(Some(credentials.clone())).build();
        let (event_tx, event_rx) = kanal::unbounded_async();
        let market_url = Url::parse("wss://fstream.binance.com/ws").unwrap();
        let market_ws = MarketWsManager::new(market_url, event_tx.clone(), 1000).await;
        let user_url = Url::parse("wss://fstream.binance.com").unwrap();
        let user_ws = UserWsManager::new(user_url, listen_key, event_tx, 1000).await;
        Self { http_client, market_ws, user_ws, event_rx }
        Self {
            venue,
            ws_manager,
            http_client,
            credentials,
        }
        .into()
    }

    async fn place_order(&self, ctx: Arc<CoreCtx>, order: &mut VenueOrder) -> Result<()> {
        info!(target: "executor-binance", "received new order: {}", order.id);

        let request: NewOrderRequest = order.clone().into();

        info!(target: "executor-binance", "sending new order request for order {}", order.id);
        let binance_order: BinanceOrder = self.ws_manager.request("trade.order.place", Some(request)).await?;

        info!(
            target: "executor-binance",
            "successfully placed order {}, Binance ID: {}",
            order.id,
            binance_order.order_id
        );
        order.place(ctx.now().await);
        order.venue_order_id = Some(binance_order.order_id.to_string());
        ctx.publish(Event::VenueOrderPlaced(order.clone().into())).await;

        Ok(())
    }

    async fn cancel_order(&self, ctx: Arc<CoreCtx>, order: &VenueOrder) -> Result<()> {
        info!(target: "executor-binance", "received cancel order for {}", order.id);

        let request = CancelOrderRequest {
            symbol: order.instrument.symbol.clone(),
            order_id: order.venue_order_id.as_ref().and_then(|id| id.parse::<i64>().ok()),
            orig_client_order_id: None, // Or use client order id if available
        };

        let cancelled_order: BinanceOrder = self.ws_manager.request("trade.order.cancel", Some(request)).await?;

        info!(
            target: "executor-binance",
            "successfully cancelled order {}, Binance ID: {}",
            order.id,
            cancelled_order.order_id
        );
        // TODO: This is not right, we should probably have a separate cancelled event
        // and update the order status accordingly.
        // For now, just publishing the original order.
        ctx.publish(Event::CancelVenueOrder(order.clone().into())).await;

        Ok(())
    }

    async fn cancel_all(&self, _ctx: Arc<CoreCtx>) {
        info!(target: "executor-binance", "received cancel all orders");
        // TODO: Implementation
    }

    async fn get_account_info(&self) {
        info!(target: "executor-binance", "request account info");
        // let req = AccountRequest::new();
        // let res = self.http_client.send(req).await;
        // match res {
        //     Ok(v) => v,
        //     Err(e) => e,
        // }
        // // Parse and send on the publish
    }

    async fn get_open_orders(&self) {
        info!(target: "executor-binance", "request open orders");
    }

    async fn handle_ws_event(&self, event: WsEvent) {
        match event {
            WsEvent::AggTrade(trade) => {
                debug!(target: "executor-binance", "Received agg trade: {:?}", trade);
            }
            WsEvent::BookTicker(ticker) => {
                debug!(target: "executor-binance", "Received book ticker: {:?}", ticker);
            }
            WsEvent::AccountUpdate(update) => {
                info!(target: "executor-binance", "Received account update: {:?}", update);
            }
            WsEvent::OrderTradeUpdate(update) => {
                info!(target: "executor-binance", "Received order trade update: {:?}", update);
            }
        }
    }
}

#[async_trait]
impl Runnable for BinanceExecutor {
    async fn handle_event(&self, ctx: Arc<CoreCtx>, event: Event) {
        let result = match event {
            Event::NewVenueOrder(o) => {
                let mut order = (*o).clone();
                self.place_order(ctx, &mut order).await
            }
            Event::CancelVenueOrder(o) => self.cancel_order(ctx, &o).await,
            Event::CancelAllVenueOrders(_) => {
                self.cancel_all(ctx).await;
                Ok(())
            }
            e => {
                warn!(target: "executor-binance", "received unused event {}", e);
                Ok(())
            }
        };

        if let Err(e) = result {
            error!(target: "executor-binance", "error handling event: {}", e);
        }
    }

    async fn setup(&self, _service_ctx: Arc<ServiceCtx>, _core_ctx: Arc<CoreCtx>) {
        info!(target: "executor-binance", "starting up...");

        // 1. Fetch initial state
        // match self.get_account_info().await {
        // Ok(info) => info!(target: "executor-binance", "Initial account info: {:?}", info),
        // Err(e) => error!(target: "executor-binance", "Failed to get account info: {}", e),
        // }
        // match self.get_open_orders().await {
        // Ok(orders) => info!(target: "executor-binance", "Initial open orders: {:?}", orders),
        // Err(e) => error!(target: "executor-binance", "Failed to get open orders: {}", e),
        // }

        // 2. Authenticate WebSocket
        info!(target: "executor-binance", "Authenticating WebSocket...");
        let params = self.credentials.ws_auth_params();
        match self
            .ws_manager
            .request::<_, EmptyResult>("api.authenticate", Some(params))
            .await
        {
            Ok(_) => info!(target: "executor-binance", "WebSocket authentication successful."),
            Err(e) => {
                error!(target: "executor-binance", "WebSocket authentication failed: {}", e);
                // Decide if we should panic or retry
                return;
            }
        }

        // 3. Subscribe to streams
        info!(target: "executor-binance", "Subscribing to WebSocket streams...");
        let streams = vec![
            // TODO: Get symbols from venue config
            "btcusdt@aggTrade".to_string(),
            "btcusdt@bookTicker".to_string(),
        ];
        if let Err(e) = self.ws_manager.subscribe(streams).await {
            error!(target: "executor-binance", "Failed to subscribe to market data streams: {}", e);
        }

        // 4. Start listening to WebSocket events
        // let (tx, mut rx) = kanal::unbounded_async();
        // This is a bit of a hack. We need to replace the WsManager with one that has the event sender.
        // A better approach would be to pass the sender into the WsManager constructor.
        // For now, we'll just log a warning.
        warn!(target: "executor-binance", "Event listener not fully implemented yet. Re-architecting WsManager is needed.");

        // let self_clone = self.clone();
        // tokio::spawn(async move {
        //     while let Some(event) = rx.recv().await {
        //         self_clone.handle_ws_event(event).await;
        //     }
        // });
    }
}

impl From<VenueOrder> for NewOrderRequest {
    fn from(order: VenueOrder) -> Self {
        match order.order_type {
            VenueOrderType::Market => NewOrderRequest {
                symbol: order.instrument.symbol.clone(),
                side: order.side.into(),
                order_type: BinanceOrderType::Market,
                quantity: Some(order.quantity),
                ..Default::default()
            },
            VenueOrderType::Limit => NewOrderRequest {
                symbol: order.instrument.symbol.clone(),
                side: order.side.into(),
                order_type: BinanceOrderType::Limit,
                quantity: Some(order.quantity),
                price: Some(order.price),
                time_in_force: Some(BinanceTimeInForce::Gtc),
                ..Default::default()
            },
            _ => panic!("Unsupported order type for conversion"),
        }
    }
}
