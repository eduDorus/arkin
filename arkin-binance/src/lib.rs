#[allow(dead_code, unused)]
mod common;

#[allow(dead_code, unused)]
mod derivatives_trading_usds_futures;

#[cfg(test)]
mod tests {
    use std::{env, time::Duration};

    use anyhow::{Context, Result};
    use rust_decimal::prelude::*;
    use tracing::info;
    use uuid::Uuid;

    use crate::{
        common::{
            config::{ConfigurationWebsocketApi, ConfigurationWebsocketStreams},
            models::WebsocketMode,
            utils::SignatureGenerator,
        },
        derivatives_trading_usds_futures::{
            websocket_api::{
                CancelOrderParams, NewOrderParams, NewOrderSideEnum, NewOrderTimeInForceEnum, StartUserDataStreamParams,
            },
            websocket_streams::IndividualSymbolBookTickerStreamsParams,
            DerivativesTradingUsdsFuturesWsApi, DerivativesTradingUsdsFuturesWsStreams,
        },
    };

    #[tokio::test]
    #[test_log::test]
    async fn subscribe_binance_agg_trades() -> Result<()> {
        // Build WebSocket Streams config
        let ws_streams_conf = ConfigurationWebsocketStreams::builder().build();

        // Create the DerivativesTradingUsdsFutures WebSocket Streams client
        let ws_streams_client = DerivativesTradingUsdsFuturesWsStreams::production(ws_streams_conf);

        // Connect to WebSocket
        let connection = ws_streams_client
            .connect()
            .await
            .context("Failed to connect to WebSocket Streams")?;

        // Setup the stream parameters
        let params = IndividualSymbolBookTickerStreamsParams::builder()
            .symbol("btcusdt".to_string())
            .build();

        // Subscribe to the stream
        let stream = connection
            .individual_symbol_book_ticker_streams(params)
            .await
            .context("Failed to subscribe to the stream")?;

        // Register callback for incoming messages
        stream.on_message(|data| {
            info!("{:?}", data);
        });

        // Disconnect after 20 seconds
        tokio::time::sleep(Duration::from_secs(20)).await;
        connection.disconnect().await.context("Failed to disconnect WebSocket client")?;

        Ok(())
    }

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
