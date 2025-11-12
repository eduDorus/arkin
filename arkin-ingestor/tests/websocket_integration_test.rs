use arkin_ingestor::registry::{WsChannel, MAPPINGS};
use arkin_ingestor::ws::{WsClient, WsConfig};
use arkin_ingestor::IngestorConfig;
use std::time::Duration;
use tokio::time::timeout;
use tracing::info;

use arkin_core::prelude::*;

#[tokio::test]
#[test_log::test]
async fn test_ws_binance_spot_agg_trades() {
    info!("Testing Binance Spot AggTrades channel");

    let config = load::<IngestorConfig>();
    info!("Loaded IngestorConfig: {:?}", config);

    // // Build subscription for AggTrades channel
    // let instruments = vec!["BTCUSDT".to_string()];
    // let sub_json = mapping
    //     .build_subscription_json(&[WsChannel::AggTrades], &instruments)
    //     .expect("Failed to build subscription");

    // info!("Subscription message: {}", sub_json);

    // // Create WS config
    // let ws_config = WsConfig {
    //     url: mapping.ws_url.to_string(),
    //     streams: vec![(sub_json, "binance_spot_agg_trades".to_string())],
    //     reconnect_backoff_ms: 1000,
    //     max_reconnect_backoff_ms: 5000,
    //     ping_interval_secs: 30,
    //     stale_connection_timeout_secs: 60,
    // };

    // // Create WS client
    // let (mut client, mut receiver) = WsClient::new(ws_config);

    // // Start the websocket client and collect messages
    // let test_task = tokio::spawn(async move {
    //     let mut message_count = 0;
    //     let max_messages = 5;

    //     loop {
    //         match timeout(Duration::from_secs(10), receiver.recv()).await {
    //             Ok(Some(msg)) => {
    //                 message_count += 1;
    //                 info!("Received message {}: {:?}", message_count, msg);

    //                 if message_count >= max_messages {
    //                     break;
    //                 }
    //             }
    //             Ok(None) => {
    //                 info!("Channel closed");
    //                 break;
    //             }
    //             Err(_) => {
    //                 info!("Timeout waiting for messages");
    //                 break;
    //             }
    //         }
    //     }

    //     message_count
    // });

    // let client_task = tokio::spawn(async move {
    //     let _ = client.run().await;
    // });

    // // Wait for test to complete or timeout
    // let result = timeout(Duration::from_secs(30), test_task).await;

    // // Cancel the client task
    // client_task.abort();

    // match result {
    //     Ok(Ok(count)) => {
    //         info!("✓ Successfully received {} messages from Binance Spot AggTrades", count);
    //         assert!(count > 0, "Expected to receive at least 1 message");
    //     }
    //     Ok(Err(e)) => {
    //         panic!("Test task failed: {}", e);
    //     }
    //     Err(_) => {
    //         panic!("Test timeout - no messages received within 30 seconds");
    //     }
    // }
}
