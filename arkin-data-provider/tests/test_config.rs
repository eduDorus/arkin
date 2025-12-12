use arkin_core::prelude::MockPersistence;
use arkin_data_provider::{config::DataProviderConfig, ProviderFactory};
use std::time::Duration;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::info;

#[tokio::test]
#[test_log::test]
async fn test_config_structure() {
    let yaml = r#"
data_providers:
  - type: binance_spot
    http_url: "https://api.binance.com"
    ws_url: "wss://stream.binance.com:9443/ws"
    ws_endpoints:
      - channel: agg_trades
        enabled: true
        prefix: ""
        suffix: "@aggTrade"
        symbols:
          - "btcusdt"
          - "ethusdt"
          - "solusdt"
      - channel: ticker
        enabled: true
        prefix: ""
        suffix: "@bookTicker"
        symbols:
          - "btcusdt"
          - "ethusdt"
          - "solusdt"
      - channel: trades
        enabled: true
        prefix: ""
        suffix: "@trade"
        symbols:
          - "btcusdt"
          - "ethusdt"
          - "solusdt"
  - type: binance_perpetual
    http_url: "https://api.binance.com"
    ws_url: "wss://fstream.binance.com/ws"
    ws_endpoints:
      - channel: agg_trades
        enabled: true
        prefix: ""
        suffix: "@aggTrade"
        symbols:
          - "btcusdt"
          - "ethusdt"
          - "solusdt"
      - channel: ticker
        enabled: true
        prefix: ""
        suffix: "@bookTicker"
        symbols:
          - "btcusdt"
          - "ethusdt"
          - "solusdt"
"#;

    let persistence = MockPersistence::new();
    let config: DataProviderConfig = serde_yaml::from_str(yaml).expect("Failed to parse config");
    assert_eq!(config.data_providers.len(), 2);
    let data_providers = ProviderFactory::from_config(config, persistence.clone());

    // Start Providers
    info!("Starting WebSocket Clients...");
    let task_tracker = TaskTracker::new();
    let shutdown = CancellationToken::new();

    let (tx, rx) = kanal::unbounded_async();
    for mut client in data_providers.1 {
        let ws_shutdown = shutdown.clone();
        let tx = tx.clone();
        // Run the client in a separate task to avoid blocking
        task_tracker.spawn(async move {
            client.run(tx, ws_shutdown).await;
        });
    }

    let rx_shutdown = shutdown.clone();
    task_tracker.spawn(async move {
        loop {
            tokio::select! {
                Ok(msg) = rx.recv() => {
                  info!("Received message from WS client: {:?}", msg);
                }
                _ = rx_shutdown.cancelled() => {
                  info!("Shutdown signal received, stopping WS client listener task");
                    break;
                }
            }
        }
    });

    // Let it run for a short duration then cancel
    tokio::time::sleep(Duration::from_secs(5)).await;
    shutdown.cancel();
    task_tracker.close();
    task_tracker.wait().await;
    info!("All tasks have been shut down.");
}
