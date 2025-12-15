use arkin_core::prelude::*;
use arkin_data_provider::ws_providers::binance_spot_user::BinanceSpotUserWsProvider;
use arkin_data_provider::WebSocketProvider;
use url::Url;

#[tokio::test]
async fn test_spot_parse_account_position() {
    let mock_persistence = MockPersistence::new();

    let provider = BinanceSpotUserWsProvider::builder()
        .api_key("test_key".to_string())
        .api_secret("test_secret".to_string())
        .http_url(Url::parse("https://api.binance.com").unwrap())
        .ws_url(Url::parse("wss://stream.binance.com:9443/ws").unwrap())
        .persistence(mock_persistence)
        .build();

    let msg = r#"{
        "e": "outboundAccountPosition",
        "E": 1690474245000,
        "u": 1690474245000,
        "B": [
            {
                "a": "USDT",
                "f": "1000.00000000",
                "l": "0.00000000"
            },
            {
                "a": "BTC",
                "f": "0.50000000",
                "l": "0.10000000"
            }
        ]
    }"#;

    let event = provider.parse(msg).await.expect("Parse failed");

    assert!(event.is_some());
    if let Some(Event::VenueAccountUpdate(update)) = event {
        assert_eq!(update.balances.len(), 2);
        assert_eq!(update.positions.len(), 0); // Spot doesn't have positions
        assert_eq!(update.reason, "ACCOUNT_UPDATE");
    } else {
        panic!("Expected VenueAccountUpdate event");
    }
}

#[tokio::test]
async fn test_spot_parse_balance_update() {
    let mock_persistence = MockPersistence::new();

    let provider = BinanceSpotUserWsProvider::builder()
        .api_key("test_key".to_string())
        .api_secret("test_secret".to_string())
        .http_url(Url::parse("https://api.binance.com").unwrap())
        .ws_url(Url::parse("wss://stream.binance.com:9443/ws").unwrap())
        .persistence(mock_persistence)
        .build();

    let msg = r#"{
        "e": "balanceUpdate",
        "E": 1690474245000,
        "a": "USDT",
        "d": "100.00000000",
        "T": 1690474245000
    }"#;

    let event = provider.parse(msg).await.expect("Parse failed");

    assert!(event.is_some());
    if let Some(Event::VenueAccountUpdate(update)) = event {
        assert_eq!(update.balances.len(), 1);
        assert_eq!(update.positions.len(), 0);
        assert_eq!(update.reason, "BALANCE_UPDATE");
        assert_eq!(update.balances[0].quantity_change, rust_decimal::Decimal::new(100, 0));
    } else {
        panic!("Expected VenueAccountUpdate event");
    }
}

#[tokio::test]
async fn test_spot_parse_execution_report() {
    let mock_persistence = MockPersistence::new();

    let provider = BinanceSpotUserWsProvider::builder()
        .api_key("test_key".to_string())
        .api_secret("test_secret".to_string())
        .http_url(Url::parse("https://api.binance.com").unwrap())
        .ws_url(Url::parse("wss://stream.binance.com:9443/ws").unwrap())
        .persistence(mock_persistence)
        .build();

    let msg = r#"{
        "e": "executionReport",
        "E": 1690474245000,
        "s": "BTCUSDT",
        "c": "testOrder",
        "S": "BUY",
        "o": "LIMIT",
        "f": "GTC",
        "q": "1.00000000",
        "p": "50000.00000000",
        "P": "0.00000000",
        "F": "0.00000000",
        "g": 0,
        "C": null,
        "x": "NEW",
        "X": "NEW",
        "r": "NONE",
        "i": 123456789,
        "l": "0.00000000",
        "z": "0.00000000",
        "L": "0.00000000",
        "n": "0.00000000",
        "N": null,
        "T": 1690474245000,
        "t": 0,
        "I": 123456789,
        "w": true,
        "m": false,
        "M": false,
        "O": 1690474245000,
        "Z": "0.00000000",
        "Y": "0.00000000",
        "Q": "0.00000000",
        "W": 1690474245000,
        "V": "NONE"
    }"#;

    let event = provider.parse(msg).await.expect("Parse failed");

    assert!(event.is_none());
}
