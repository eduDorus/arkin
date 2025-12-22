use std::sync::Arc;

use arkin_core::prelude::*;
use arkin_core::test_utils;
use arkin_exec_binance::client::BinanceClient;
use arkin_exec_binance::config::BinanceMarginExecutionConfig;
use arkin_exec_binance::config::{
    BinanceExecutionConfig, BinanceExecutionServiceConfig, BinanceSpotExecutionConfig, BinanceUsdmExecutionConfig,
};
use arkin_exec_binance::BinanceMarketType;
use rust_decimal::prelude::*;
use time::UtcDateTime;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

async fn create_test_setup() -> (Arc<CoreCtx>, MockServer) {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Create test dependencies
    let time = MockTime::new();
    let pubsub = test_pubsub();
    let persistence = MockPersistence::new();
    let core_ctx = Arc::new(CoreCtx::new(time, pubsub, persistence));

    (core_ctx, mock_server)
}

fn create_test_config(spot_url: &str, usdm_url: &str) -> BinanceExecutionServiceConfig {
    // Remove trailing slash from URLs if present
    let spot_url = spot_url.trim_end_matches('/');
    let usdm_url = usdm_url.trim_end_matches('/');

    BinanceExecutionServiceConfig {
        binance_execution: BinanceExecutionConfig {
            spot: Some(BinanceSpotExecutionConfig {
                enabled: true,
                api_key: "test_key".to_string(),
                api_secret: "test_secret".to_string(),
                base_url: spot_url.parse().unwrap(),
                testnet: true,
            }),
            margin: Some(BinanceMarginExecutionConfig {
                enabled: true,
                api_key: "test_key".to_string(),
                api_secret: "test_secret".to_string(),
                base_url: spot_url.parse().unwrap(),
                testnet: true,
            }),
            usdm: Some(BinanceUsdmExecutionConfig {
                enabled: true,
                api_key: "test_key".to_string(),
                api_secret: "test_secret".to_string(),
                base_url: usdm_url.parse().unwrap(),
                testnet: true,
            }),
        },
    }
}

fn create_spot_order() -> Arc<VenueOrder> {
    // Create a proper test instrument
    let instrument = test_utils::test_inst_binance_btc_usdt_spot();

    Arc::new(
        VenueOrder::builder()
            .execution_order_id(uuid::Uuid::new_v4())
            .instrument(instrument)
            .side(MarketSide::Buy)
            .order_type(VenueOrderType::Limit)
            .time_in_force(VenueOrderTimeInForce::Gtc)
            .set_price(Decimal::new(50000, 0))
            .set_quantity(Decimal::new(1, 0))
            .strategy(Some(test_strategy_1()))
            .created(UtcDateTime::now())
            .updated(UtcDateTime::now())
            .build(),
    )
}

fn create_usdm_order() -> Arc<VenueOrder> {
    // Create a proper test instrument
    let instrument = test_utils::test_inst_binance_btc_usdt_perp();

    Arc::new(
        VenueOrder::builder()
            .execution_order_id(uuid::Uuid::new_v4())
            .instrument(instrument)
            .side(MarketSide::Buy)
            .order_type(VenueOrderType::Limit)
            .time_in_force(VenueOrderTimeInForce::Gtc)
            .set_price(Decimal::new(50000, 0))
            .set_quantity(Decimal::new(1, 0))
            .strategy(Some(test_strategy_1()))
            .created(UtcDateTime::now())
            .updated(UtcDateTime::now())
            .build(),
    )
}

#[tokio::test]
async fn test_spot_order_placement() {
    let (_core_ctx, mock_server) = create_test_setup().await;

    // Mock the Binance Spot API response
    Mock::given(method("POST"))
        .and(path("/api/v3/order"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "orderId": 12345,
            "clientOrderId": "test123",
            "symbol": "BTCUSDT",
            "status": "NEW"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    println!("Mock server URI: {}", mock_server.uri());
    println!("Mock mounted for POST /api/v3/order");

    let config = create_test_config(&mock_server.uri(), "https://testnet.binancefuture.com");
    let client = BinanceClient::new(config.binance_execution);
    let order = create_spot_order();

    println!("Order instrument type: {:?}", order.instrument.instrument_type);
    println!("Order venue symbol: {}", order.instrument.venue_symbol);

    // Place order directly via client
    let result = client.place_order(&order).await;
    if let Err(e) = &result {
        println!("Spot order placement error: {:?}", e);
    }
    assert!(result.is_ok());

    // Verify the mock was called
    mock_server.verify().await;
}

#[tokio::test]
async fn test_spot_order_cancellation() {
    let (_core_ctx, mock_server) = create_test_setup().await;

    // Mock the Binance Spot cancel API response
    Mock::given(method("DELETE"))
        .and(path("/api/v3/order"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "orderId": 12345,
            "clientOrderId": "test123",
            "status": "CANCELED"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = create_test_config(&mock_server.uri(), "https://testnet.binancefuture.com");
    let client = BinanceClient::new(config.binance_execution);

    // Create order with venue_order_id set
    let mut order = create_spot_order();
    // Set venue_order_id to simulate an already placed order
    Arc::get_mut(&mut order).unwrap().venue_order_id = Some("12345".to_string());

    // Cancel order directly via client
    let result = client.cancel_order(&order).await;
    assert!(result.is_ok());

    // Verify the mock was called
    mock_server.verify().await;
}

#[tokio::test]
async fn test_spot_cancel_all_orders() {
    let (_core_ctx, mock_server) = create_test_setup().await;

    // Mock the Binance Spot cancel all API response
    Mock::given(method("DELETE"))
        .and(path("/api/v3/openOrders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
            {
                "orderId": 12345,
                "clientOrderId": "test123",
                "status": "CANCELED"
            }
        ])))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = create_test_config(&mock_server.uri(), "https://testnet.binancefuture.com");
    let client = BinanceClient::new(config.binance_execution);

    // Cancel all orders directly via client
    let result = client.cancel_all_orders(Some("BTCUSDT"), BinanceMarketType::Spot).await;
    assert!(result.is_ok());

    // Verify the mock was called
    mock_server.verify().await;
}

#[tokio::test]
async fn test_usdm_order_placement() {
    let (_core_ctx, mock_server) = create_test_setup().await;

    // Mock the Binance USDM API response
    Mock::given(method("POST"))
        .and(path("/fapi/v1/order"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "orderId": 12345,
            "clientOrderId": "test123",
            "status": "NEW"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = create_test_config("https://testnet.binance.com", &mock_server.uri());
    let client = BinanceClient::new(config.binance_execution);
    let order = create_usdm_order();

    // Place order directly via client
    let result = client.place_order(&order).await;
    assert!(result.is_ok());

    // Verify the mock was called
    mock_server.verify().await;
}

#[tokio::test]
async fn test_usdm_order_cancellation() {
    let (_core_ctx, mock_server) = create_test_setup().await;

    // Mock the Binance USDM cancel API response
    Mock::given(method("DELETE"))
        .and(path("/fapi/v1/order"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "orderId": 12345,
            "clientOrderId": "test123",
            "status": "CANCELED"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = create_test_config("https://testnet.binance.com", &mock_server.uri());
    let client = BinanceClient::new(config.binance_execution);

    // Create order with venue_order_id set
    let mut order = create_usdm_order();
    // Set venue_order_id to simulate an already placed order
    Arc::get_mut(&mut order).unwrap().venue_order_id = Some("12345".to_string());

    // Cancel order directly via client
    let result = client.cancel_order(&order).await;
    assert!(result.is_ok());

    // Verify the mock was called
    mock_server.verify().await;
}

#[tokio::test]
async fn test_usdm_cancel_all_orders() {
    let (_core_ctx, mock_server) = create_test_setup().await;

    // Mock the Binance USDM cancel all API response
    Mock::given(method("DELETE"))
        .and(path("/fapi/v1/allOpenOrders"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 200,
            "msg": "success"
        })))
        .expect(1)
        .mount(&mock_server)
        .await;

    let config = create_test_config("https://testnet.binance.com", &mock_server.uri());
    let client = BinanceClient::new(config.binance_execution);

    // Cancel all orders directly via client
    let result = client.cancel_all_orders(Some("BTCUSDT"), BinanceMarketType::Usdm).await;
    assert!(result.is_ok());

    // Verify the mock was called
    mock_server.verify().await;
}
