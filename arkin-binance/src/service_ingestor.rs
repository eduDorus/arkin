use std::sync::Arc;

use async_trait::async_trait;
use tracing::info;
use typed_builder::TypedBuilder;

use arkin_core::prelude::*;

use crate::{
    common::config::ConfigurationWebsocketStreams,
    derivatives_trading_usds_futures::{
        websocket_streams::IndividualSymbolBookTickerStreamsParams, DerivativesTradingUsdsFuturesWsStreams,
    },
};

#[derive(TypedBuilder)]
pub struct BinanceIngestor {
    identifier: String,
    _time: Arc<dyn SystemTime>,
    _publisher: Arc<dyn Publisher>,
}

#[async_trait]
impl Runnable for BinanceIngestor {
    fn identifier(&self) -> &str {
        &self.identifier
    }

    async fn start_tasks(self: Arc<Self>, ctx: Arc<ServiceCtx>) {
        info!(target: "ingestor::binance", "starting simulation tasks");

        // let _publisher = self._publisher.clone();
        let shutdown = ctx.get_shutdown_token();

        // Build WebSocket Streams config
        let ws_streams_conf = ConfigurationWebsocketStreams::builder().build();

        // Create the DerivativesTradingUsdsFutures WebSocket Streams client
        let ws_streams_client = DerivativesTradingUsdsFuturesWsStreams::production(ws_streams_conf);

        // Connect to WebSocket
        let connection = ws_streams_client.connect().await.unwrap();

        ctx.spawn(async move {
            // Setup the stream parameters
            let params = IndividualSymbolBookTickerStreamsParams::builder()
                .symbol("btcusdt".to_string())
                .build();

            // Subscribe to the stream
            let stream = connection.individual_symbol_book_ticker_streams(params).await.unwrap();
            // Register callback for incoming messages
            stream.on_message(|data| {
                info!("{:?}", data);
                // Here we will publish
            });

            shutdown.cancelled().await;
            info!(target: "ingestor::binance", "binance sim ingestor received shutdown");
            connection.disconnect().await.unwrap();
            info!(target: "ingestor::binance", "binance sim ingestor finished task");
        });
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use anyhow::{Context, Result};
    use tracing::info;

    use crate::{
        common::config::ConfigurationWebsocketStreams,
        derivatives_trading_usds_futures::{
            websocket_streams::IndividualSymbolBookTickerStreamsParams, DerivativesTradingUsdsFuturesWsStreams,
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
}
