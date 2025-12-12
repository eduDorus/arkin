// use std::time::Duration;

// use url::Url;

// use arkin_core::prelude::*;
// use arkin_data_provider::prelude::*;

// #[tokio::test]
// #[test_log::test]
// async fn test_binance_spot_ws_client_connection() {
//     let provider = BinanceSpotWsProvider::builder()
//         .url(Url::parse("wss://stream.binance.com:9443/ws").unwrap())
//         .channel(Channel::AggTrades)
//         .symbols(vec!["btcusdt".to_string()])
//         .build();

//     let mut client = WsClient::new(Box::new(provider));

//     // Run the client in a separate task to avoid blocking
//     let client_task = tokio::spawn(async move {
//         client.run().await.unwrap();
//     });

//     // Let it run for a short duration then cancel
//     tokio::time::sleep(Duration::from_secs(10)).await;
//     client_task.abort();
// }

// #[tokio::test]
// #[test_log::test]
// async fn test_binance_usdm_ws_client_connection() {
//     let provider = BinanceUsdmWsProvider::builder()
//         .url(Url::parse("wss://fstream.binance.com/ws").unwrap())
//         .channel(Channel::AggTrades)
//         .symbols(vec!["btcusdt".to_string()])
//         .build();

//     let mut client = WsClient::new(Box::new(provider));

//     // Run the client in a separate task to avoid blocking
//     let client_task = tokio::spawn(async move {
//         client.run().await.unwrap();
//     });

//     // Let it run for a short duration then cancel
//     tokio::time::sleep(Duration::from_secs(10)).await;
//     client_task.abort();
// }
