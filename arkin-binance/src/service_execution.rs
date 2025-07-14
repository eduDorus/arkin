use std::{env, sync::Arc};

use async_trait::async_trait;
use tokio::sync::RwLock;
use tracing::{info, instrument, warn};
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{
    common::{config::ConfigurationWebsocketApi, models::WebsocketMode, utils::SignatureGenerator},
    derivatives_trading_usds_futures::{
        websocket_api::{WebsocketApi, WebsocketApiHandle},
        DerivativesTradingUsdsFuturesWsApi,
    },
};

#[derive(TypedBuilder)]
pub struct BinanceExecution {
    identifier: String,
    _time: Arc<dyn SystemTime>,
    _publisher: Arc<dyn Publisher>,
    client: WebsocketApiHandle,
    connection: RwLock<Option<WebsocketApi>>,
}

impl BinanceExecution {
    pub fn new(&self, time: Arc<dyn SystemTime>, publisher: Arc<dyn Publisher>) -> Arc<Self> {
        // Load credentials from env
        let api_key = env::var("API_KEY").expect("API_KEY must be set in the environment for binance execution");
        let api_secret =
            env::var("API_SECRET").expect("API_SECRET must be set in the environment for binance execution");

        // Build WebSocket API config
        let signature_gen = SignatureGenerator::new(Some(api_secret.clone()), None, None);
        let configuration = ConfigurationWebsocketApi::builder()
            .api_key(api_key)
            .api_secret(api_secret)
            .mode(WebsocketMode::Pool(1)) // Use pool mode with a pool size of 3
            .user_agent("unknown".to_owned())
            .signature_gen(signature_gen)
            .build();

        let client = DerivativesTradingUsdsFuturesWsApi::production(configuration);

        Self {
            identifier: "execution::binance".to_owned(),
            _time: time,
            _publisher: publisher,
            client,
            connection: RwLock::new(None),
        }
        .into()
    }
    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn place_order(&self, order: &VenueOrder) {
        info!(target: "execution::binance", "received new order {}", order.id);
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn cancel_order(&self, order: &VenueOrder) {
        info!(target: "execution::binance", "received cancel order for {}", order.id);
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    pub async fn cancel_all(&self) {
        info!(target: "execution::binance", "received cancel all open orders");
    }
}

#[async_trait]
impl Runnable for BinanceExecution {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn setup(&self, _ctx: Arc<ServiceCtx>) {
        let mut guard = self.connection.write().await;
        if guard.is_none() {
            let res = self.client.connect().await;
            match res {
                Ok(conn) => {
                    info!(target: "execution::binance", "Connected successfully");
                    *guard = Some(conn);
                }
                Err(e) => info!(target: "execution::binance", "Connection error {}", e),
            }
        }
        drop(guard);
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn handle_event(&self, event: Event) {
        match &event {
            Event::NewVenueOrder(o) => self.place_order(o).await,
            Event::CancelVenueOrder(o) => self.cancel_order(o).await,
            Event::CancelAllVenueOrders(_) => self.cancel_all().await,
            e => warn!(target: "execution::binance", "received unused event {}", e),
        }
    }

    #[instrument(parent = None, skip_all, fields(service = %self.identifier()))]
    async fn teardown(&self, _ctx: Arc<ServiceCtx>) {
        let mut guard = self.connection.write().await;
        if let Some(conn) = guard.as_ref() {
            let res = conn.disconnect().await;
            match res {
                Ok(_) => {
                    info!(target: "execution::binance", "Disconnected successfully");
                    *guard = None;
                }
                Err(e) => info!(target: "execution::binance", "Disconnect error {}", e),
            }
        }
        drop(guard);
    }
}

#[cfg(test)]
mod tests {
    use std::{env, time::Duration};

    use anyhow::{Context, Result};
    use rust_decimal::prelude::*;
    use tracing::info;
    use uuid::Uuid;

    use crate::{
        common::{config::ConfigurationWebsocketApi, models::WebsocketMode, utils::SignatureGenerator},
        derivatives_trading_usds_futures::{
            websocket_api::{
                CancelOrderParams, NewOrderParams, NewOrderSideEnum, NewOrderTimeInForceEnum, StartUserDataStreamParams,
            },
            DerivativesTradingUsdsFuturesWsApi,
        },
    };

    #[tokio::test]
    #[test_log::test]
    async fn subscribe_user_stream() -> Result<()> {
        // Load credentials from env
        let api_key = env::var("API_KEY").expect("API_KEY must be set in the environment");
        let api_secret = env::var("API_SECRET").expect("API_SECRET must be set in the environment");

        // Build WebSocket API config
        let signature_gen = SignatureGenerator::new(Some(api_secret.clone()), None, None);
        let configuration = ConfigurationWebsocketApi::builder()
            .api_key(api_key)
            .api_secret(api_secret)
            .mode(WebsocketMode::Pool(1)) // Use pool mode with a pool size of 3
            .user_agent("unknown".to_owned())
            .signature_gen(signature_gen)
            .build();

        let client = DerivativesTradingUsdsFuturesWsApi::production(configuration);
        let connection = client.connect().await?;

        // Subscribe to the stream
        let _stream = connection.subscribe_on_ws_events(|e| info!("USER DATA SUBSCRIPTION STREAM: {:?}", e));

        let params = StartUserDataStreamParams::builder().build();
        let response = connection.start_user_data_stream(params).await?;
        info!(?response.rate_limits, "start_user_data_stream rate limits");
        let data = response.data()?;
        info!(?data, "start_user_data_stream data");

        // let params = AccountInformationV2Params::builder().build();
        // let response = connection.account_information_v2(params).await?;
        // let data = response.data()?;
        // info!(?data, "account information data");

        let client_order_id = Uuid::new_v4();
        let params = NewOrderParams::builder()
            .side(NewOrderSideEnum::Buy)
            .price(Some(dec!(110000)))
            .quantity(Some(dec!(0.01)))
            .r#type("LIMIT")
            .symbol("btcusdt")
            .time_in_force(Some(NewOrderTimeInForceEnum::Gtc))
            .new_client_order_id(Some(client_order_id.to_string()))
            .build();
        let response = connection.new_order(params).await?;
        let _data = response.data()?;
        // info!(?data, "new order data");

        tokio::time::sleep(Duration::from_secs(3)).await;
        let params = CancelOrderParams::builder()
            .symbol("btcusdt")
            .orig_client_order_id(client_order_id.to_string())
            .build();
        let response = connection.cancel_order(params).await?;
        let _data = response.data()?;
        // info!(?data, "cancel order data");

        // Disconnect after 10 seconds
        tokio::time::sleep(Duration::from_secs(10)).await;
        connection.disconnect().await.context("Failed to disconnect WebSocket client")?;

        Ok(())
    }
}
