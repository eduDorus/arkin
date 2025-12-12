use std::time::Duration;

use arkin_data_provider::prelude::*;
use url::Url;

#[tokio::test]
#[test_log::test]
async fn test_binance_spot_http_client_connection() {
    let provider = BinanceSpotHttpProvider::builder()
        .base_url(Url::parse("https://api.binance.com").unwrap())
        .build();

    let client = HttpExecutor::new(provider, RetryConfig::default());

    // Run the client in a separate task to avoid blocking
    let client_task = tokio::spawn(async move {
        client.run().await.unwrap();
    });

    // Let it run for a short duration then cancel
    tokio::time::sleep(Duration::from_secs(10)).await;
    client_task.abort();
}
