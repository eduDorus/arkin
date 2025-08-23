use std::env;

use arkin_core::prelude::*;
use arkin_exec_binance::{http::Credentials, service::BinanceExecutor, ws::WsManager};
use url::Url;

#[tokio::test]
#[test_log::test]
async fn test_binance_wide_quoting() {
    info!("Starting binance wide quoting test...");
    // Init mock time
    let time = MockTime::new();

    let publisher = MockPublisher::new();

    let persistence = MockPersistence::new();

    let api_key = env::var("API_KEY").expect("API_KEY must be set");
    let api_secret = env::var("API_SECRET").expect("API_SECRET must be set");
    let ws_url = Url::parse("wss://fstream.binance.com/ws").expect("Invalid base URL");
    let exec = BinanceExecutor::new(
        test_binance_venue(),
        WsManager::new(ws_url),
        Credentials::from_hmac(api_key, api_secret),
    );
    let service = Service::new("exec", exec, time.clone(), publisher.clone(), None, persistence.clone());
    service.start_service().await;
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
}
