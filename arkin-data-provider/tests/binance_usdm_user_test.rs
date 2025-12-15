use arkin_core::prelude::*;
use arkin_data_provider::ws_providers::binance_usdm_user::{BinanceUsdmUserWsProvider, UserDataEvent};
use arkin_data_provider::WebSocketProvider;
use serde_json;
use url::Url;

#[tokio::test]
async fn test_json_parsing() {
    let msg = r#"{
        "e": "ACCOUNT_UPDATE",
        "E": 1690474245000,
        "T": 1690474245000,
        "a": {
            "m": "DEPOSIT",
            "B": [
                {
                    "a": "USDT",
                    "wb": "1000.00000000",
                    "cw": "1000.00000000",
                    "bc": "1000.00000000"
                }
            ],
            "P": [
                {
                    "s": "BTCUSDT",
                    "pa": "1.00000000",
                    "ep": "50000.00000000",
                    "cr": "0.00000000",
                    "up": "1000.00000000",
                    "mt": "isolated",
                    "iw": "0.00000000",
                    "ps": "LONG"
                }
            ]
        }
    }"#;

    let event: Result<UserDataEvent, _> = serde_json::from_str(msg);
    println!("JSON parse result: {:?}", event);
    assert!(event.is_ok());
}

#[tokio::test]
async fn test_parse_account_update() {
    let mock_persistence = MockPersistence::new();

    let provider = BinanceUsdmUserWsProvider::builder()
        .api_key("test_key".to_string())
        .api_secret("test_secret".to_string())
        .http_url(Url::parse("https://fapi.binance.com").unwrap())
        .ws_url(Url::parse("wss://fstream.binance.com/ws").unwrap())
        .persistence(mock_persistence)
        .build();

    let msg = r#"{
        "e": "ACCOUNT_UPDATE",
        "E": 1690474245000,
        "T": 1690474245000,
        "a": {
            "m": "DEPOSIT",
            "B": [
                {
                    "a": "USDT",
                    "wb": "1000.00000000",
                    "cw": "1000.00000000",
                    "bc": "1000.00000000"
                }
            ],
            "P": [
                {
                    "s": "BTCUSDT",
                    "pa": "1.00000000",
                    "ep": "50000.00000000",
                    "cr": "0.00000000",
                    "up": "1000.00000000",
                    "mt": "isolated",
                    "iw": "0.00000000",
                    "ps": "LONG"
                }
            ]
        }
    }"#;

    let event = provider.parse(msg).await.expect("Parse failed");

    assert!(event.is_some());
    if let Some(Event::VenueAccountUpdate(update)) = event {
        assert_eq!(update.balances.len(), 1);
        assert_eq!(update.positions.len(), 1);
        assert_eq!(update.reason, "DEPOSIT");
    } else {
        panic!("Expected VenueAccountUpdate event");
    }
}

#[tokio::test]
async fn test_parse_margin_call() {
    let mock_persistence = MockPersistence::new();

    let provider = BinanceUsdmUserWsProvider::builder()
        .api_key("test_key".to_string())
        .api_secret("test_secret".to_string())
        .http_url(Url::parse("https://fapi.binance.com").unwrap())
        .ws_url(Url::parse("wss://fstream.binance.com/ws").unwrap())
        .persistence(mock_persistence)
        .build();

    let msg = r#"{
        "e": "MARGIN_CALL",
        "E": 1690474245000,
        "T": 1690474245000,
        "p": [
            {
                "s": "BTCUSDT",
                "pa": "1.00000000",
                "ep": "50000.00000000",
                "cr": "0.00000000",
                "up": "1000.00000000",
                "mt": "isolated",
                "iw": "0.00000000",
                "ps": "LONG"
            }
        ]
    }"#;

    let event = provider.parse(msg).await.expect("Parse failed");

    assert!(event.is_some());
    if let Some(Event::VenueAccountUpdate(update)) = event {
        assert_eq!(update.balances.len(), 0);
        assert_eq!(update.positions.len(), 1);
        assert_eq!(update.reason, "MARGIN_CALL");
    } else {
        panic!("Expected VenueAccountUpdate event");
    }
}

#[tokio::test]
async fn test_parse_order_trade_update() {
    let mock_persistence = MockPersistence::new();

    let provider = BinanceUsdmUserWsProvider::builder()
        .api_key("test_key".to_string())
        .api_secret("test_secret".to_string())
        .http_url(Url::parse("https://fapi.binance.com").unwrap())
        .ws_url(Url::parse("wss://fstream.binance.com/ws").unwrap())
        .persistence(mock_persistence)
        .build();

    let msg = r#"{
        "e": "ORDER_TRADE_UPDATE",
        "E": 1690474245000,
        "T": 1690474245000,
        "o": {
            "s": "BTCUSDT",
            "c": "testOrder",
            "S": "BUY",
            "o": "LIMIT",
            "f": "GTC",
            "q": "1.00000000",
            "p": "50000.00000000",
            "ap": "0.00000000",
            "sp": "0.00000000",
            "x": "NEW",
            "X": "NEW",
            "i": 123456789,
            "l": "0.00000000",
            "z": "0.00000000",
            "L": "0.00000000",
            "n": "0.00000000",
            "N": "USDT",
            "T": 1690474245000,
            "t": 0,
            "b": "0.00000000",
            "a": "0.00000000",
            "m": false,
            "R": false,
            "wt": "CONTRACT_PRICE",
            "ot": "LIMIT",
            "ps": "LONG",
            "cp": false,
            "AP": "0.00000000",
            "cr": "0.00000000",
            "rp": "0.00000000"
        }
    }"#;

    let event = provider.parse(msg).await.expect("Parse failed");

    println!("Parsed event: {:?}", event);
    assert!(event.is_none());
}
