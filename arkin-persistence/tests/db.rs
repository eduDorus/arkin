use std::{str::FromStr, sync::Arc};

use rstest::*;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use time::macros::datetime;

use arkin_core::prelude::*;
use arkin_persistence::prelude::*;
use uuid::Uuid;

#[fixture]
pub fn persistence_service() -> PersistenceService {
    test_setup();
    let config = load::<PersistenceConfig>();
    PersistenceService::from_config(&config)
}

// #[rstest]
// #[tokio::test]
// async fn test_insert_venue(persistence_service: PersistenceService) {
//     test_setup();
//     let venue = Venue {
//         id: Uuid::new_v4(),
//         name: "Okex".into(),
//         venue_type: "exchange".into(),
//     };
//     persistence_service.insert_venue(venue).await.unwrap();
// }

// #[rstest]
// #[tokio::test]
// async fn test_read_venue(persistence_service: PersistenceService) {
//     test_setup();
//     let id = Uuid::from_str("48adfe42-29fb-4402-888a-0204bf417e32").expect("Invalid UUID");
//     let venue = persistence_service.read_venue_by_id(id).await.unwrap().unwrap();
//     assert_eq!(venue.id, id);
//     assert_eq!(venue.name, "binance");
// }

#[rstest]
#[tokio::test]
async fn test_insert_instrument(persistence_service: PersistenceService, binance_btc_usdt_perp: Instrument) {
    test_setup();
    let mut binance_btc_usdt_perp = binance_btc_usdt_perp.clone();
    binance_btc_usdt_perp.id = Uuid::new_v4();
    persistence_service
        .insert_instrument(binance_btc_usdt_perp)
        .await
        .expect("Failed to insert instrument");
}

#[rstest]
#[tokio::test]
async fn test_read_instrument(persistence_service: PersistenceService, binance_btc_usdt_perp: Instrument) {
    test_setup();
    let id = Uuid::from_str("f5dd7db6-89da-4c68-b62e-6f80b763bef6").expect("Invalid UUID");
    let instrument = persistence_service.read_instrument_by_id(id).await.unwrap();
    assert_eq!(instrument.id, binance_btc_usdt_perp.id);
}

#[rstest]
#[tokio::test]
async fn test_insert_tick(persistence_service: PersistenceService, binance_btc_usdt_perp: Instrument) {
    test_setup();
    let instrument = Arc::new(binance_btc_usdt_perp);
    let tick = Tick::new(
        datetime!(2024-07-01 00:01).assume_utc(),
        instrument.clone(),
        1,
        Decimal::from_f64(100.0).expect("Invalid decimal"),
        Decimal::from_f64(1.0).expect("Invalid decimal"),
        Decimal::from_f64(101.0).expect("Invalid decimal"),
        Decimal::from_f64(1.0).expect("Invalid decimal"),
    );
    persistence_service.insert_tick(tick).await.unwrap();
}

#[rstest]
#[case::batch_100(100)]
#[case::batch_1000(1000)]
#[case::batch_10000(10000)]
#[case::batch_100000(100000)]
#[tokio::test]
async fn test_insert_tick_batch(
    persistence_service: PersistenceService,
    binance_btc_usdt_perp: Instrument,
    #[case] amount: i64,
) {
    test_setup();
    let instrument = Arc::new(binance_btc_usdt_perp);
    let ticks = (0..amount)
        .into_iter()
        .map(|i| {
            Tick::new(
                datetime!(2024-07-01 00:01).assume_utc() + time::Duration::seconds(i),
                instrument.clone(),
                i as u64,
                Decimal::from_f64(100.0).expect("Invalid decimal"),
                Decimal::from_f64(1.0).expect("Invalid decimal"),
                Decimal::from_f64(101.0).expect("Invalid decimal"),
                Decimal::from_f64(1.0).expect("Invalid decimal"),
            )
        })
        .collect::<Vec<_>>();
    persistence_service.insert_tick_batch_vec(ticks).await.unwrap();
}

#[rstest]
#[tokio::test]
async fn test_read_tick_range(persistence_service: PersistenceService, binance_btc_usdt_perp: Instrument) {
    test_setup();
    let from = datetime!(2024-07-01 00:00).assume_utc();
    let till = datetime!(2024-07-01 00:10).assume_utc();
    let ticks = persistence_service
        .read_ticks_range(&[Arc::new(binance_btc_usdt_perp)], from, till)
        .await
        .unwrap();
    assert!(!ticks.is_empty());
}

#[rstest]
#[tokio::test]
async fn test_insert_trade(persistence_service: PersistenceService, binance_btc_usdt_perp: Instrument) {
    test_setup();
    let instrument = Arc::new(binance_btc_usdt_perp);
    let trade = Trade::new(
        datetime!(2024-07-01 00:01).assume_utc(),
        instrument,
        1,
        MarketSide::Buy,
        Decimal::from_f64(100.0).expect("Invalid decimal"),
        Decimal::from_f64(4.3).expect("Invalid decimal"),
    );
    persistence_service.insert_trade(trade).await.unwrap();
}

#[rstest]
#[case::batch_100(100)]
#[case::batch_1000(1000)]
#[case::batch_10000(10000)]
#[case::batch_100000(100000)]
#[tokio::test]
async fn test_insert_trade_batch(
    persistence_service: PersistenceService,
    binance_btc_usdt_perp: Instrument,
    #[case] amount: i64,
) {
    use std::sync::Arc;

    test_setup();
    let instrument = Arc::new(binance_btc_usdt_perp);
    let trades = (0..amount)
        .into_iter()
        .map(|i| {
            Trade::new(
                datetime!(2024-07-01 00:01).assume_utc() + time::Duration::seconds(i),
                instrument.clone(),
                i as u64,
                MarketSide::Buy,
                Decimal::from_f64(100.0).expect("Invalid decimal"),
                Decimal::from_f64(4.3).expect("Invalid decimal"),
            )
        })
        .collect::<Vec<_>>();
    persistence_service.insert_trade_batch_vec(trades).await.unwrap();
}

#[rstest]
#[tokio::test]
async fn test_read_trade_range(persistence_service: PersistenceService, binance_btc_usdt_perp: Instrument) {
    test_setup();
    let from = datetime!(2024-07-01 00:00).assume_utc();
    let till = datetime!(2024-07-01 00:10).assume_utc();
    let trades = persistence_service
        .read_trades_range(&[Arc::new(binance_btc_usdt_perp)], from, till)
        .await
        .unwrap();
    assert!(!trades.is_empty());
}
