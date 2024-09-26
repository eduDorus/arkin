use std::str::FromStr;

use rstest::*;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use time::macros::datetime;

use arkin_core::prelude::*;
use arkin_persistance::prelude::*;
use uuid::Uuid;

#[fixture]
pub fn database() -> PersistanceManager {
    test_setup();
    let config = load::<PersistanceConfig>();
    PersistanceManager::from_config(&config.database)
}

#[rstest]
#[tokio::test]
async fn test_insert_tick(database: PersistanceManager, binance_btc_usdt_perp: Instrument) {
    test_setup();
    let tick = Tick::new(
        datetime!(2024-07-01 00:01).assume_utc(),
        binance_btc_usdt_perp,
        1,
        Decimal::from_f64(100.0).expect("Invalid decimal"),
        Decimal::from_f64(1.0).expect("Invalid decimal"),
        Decimal::from_f64(101.0).expect("Invalid decimal"),
        Decimal::from_f64(1.0).expect("Invalid decimal"),
    );
    let db_tick = DBTick::from(tick);
    db_tick.insert(&database.pool).await.unwrap();
}

#[rstest]
#[case::batch_100(100)]
#[case::batch_1000(1000)]
#[case::batch_10000(10000)]
#[case::batch_100000(100000)]
#[tokio::test]
async fn test_insert_tick_batch(database: PersistanceManager, binance_btc_usdt_perp: Instrument, #[case] amount: i64) {
    test_setup();
    let ticks = (0..amount)
        .into_iter()
        .map(|i| {
            Tick::new(
                datetime!(2024-07-01 00:01).assume_utc() + time::Duration::seconds(i),
                binance_btc_usdt_perp.clone(),
                i as u64,
                Decimal::from_f64(100.0).expect("Invalid decimal"),
                Decimal::from_f64(1.0).expect("Invalid decimal"),
                Decimal::from_f64(101.0).expect("Invalid decimal"),
                Decimal::from_f64(1.0).expect("Invalid decimal"),
            )
        })
        .collect::<Vec<_>>();
    let tick_repo = database.tick_repo();
    tick_repo.insert_batch(&database.pool, ticks).await.unwrap();
}

#[rstest]
#[tokio::test]
async fn test_read_tick_range(database: PersistanceManager) {
    test_setup();
    let from = datetime!(2024-07-01 00:00).assume_utc();
    let till = datetime!(2024-07-01 00:10).assume_utc();
    let tick_repo = database.tick_repo();
    let ticks = tick_repo.read_range(&database.pool, from, till).await.unwrap();
    assert_eq!(ticks.len(), 1);
}

#[rstest]
#[tokio::test]
async fn test_insert_trade(database: PersistanceManager, binance_btc_usdt_perp: Instrument) {
    test_setup();
    let trade_repo = database.trade_repo();
    for i in 0..100 {
        let trade = Trade::new(
            datetime!(2024-07-01 00:01).assume_utc() + time::Duration::seconds(i),
            binance_btc_usdt_perp.clone(),
            i as u64,
            Decimal::from_f64(100.0).expect("Invalid decimal"),
            Decimal::from_f64(4.3).expect("Invalid decimal"),
        );
        trade_repo.insert(&database.pool, trade).await.unwrap();
    }
}

#[rstest]
#[case::batch_100(100)]
#[case::batch_1000(1000)]
#[case::batch_10000(10000)]
#[case::batch_100000(100000)]
#[tokio::test]
async fn test_insert_trade_batch(database: PersistanceManager, binance_btc_usdt_perp: Instrument, #[case] amount: i64) {
    test_setup();
    let trade_repo = database.trade_repo();
    let trades = (0..amount)
        .into_iter()
        .map(|i| {
            Trade::new(
                datetime!(2024-07-01 00:01).assume_utc() + time::Duration::seconds(i),
                binance_btc_usdt_perp.clone(),
                i as u64,
                Decimal::from_f64(100.0).expect("Invalid decimal"),
                Decimal::from_f64(4.3).expect("Invalid decimal"),
            )
        })
        .collect::<Vec<_>>();
    trade_repo.insert_batch(&database.pool, trades).await.unwrap();
}

#[rstest]
#[tokio::test]
async fn test_insert_venue(database: PersistanceManager) {
    test_setup();
    let venue_repo = database.venue_repo();
    let venue = Venue {
        id: Uuid::new_v4(),
        name: "Okex".into(),
        venue_type: "exchange".into(),
    };
    venue_repo.insert(&database.pool, venue.clone()).await.unwrap();
}

#[rstest]
#[tokio::test]
async fn test_read_venue(database: PersistanceManager) {
    test_setup();
    let venue_repo = database.venue_repo();
    let venue = venue_repo
        .read_by_id(
            &database.pool,
            Uuid::from_str("48adfe42-29fb-4402-888a-0204bf417e32").expect("Invalid UUID"),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        venue.id,
        Uuid::from_str("48adfe42-29fb-4402-888a-0204bf417e32").expect("Invalid UUID")
    );
    assert_eq!(venue.name, "binance");
}

#[rstest]
#[tokio::test]
async fn test_insert_instrument(database: PersistanceManager, binance_btc_usdt_perp: Instrument) {
    test_setup();
    let instrument_repo = database.instrument_repo();
    let mut binance_btc_usdt_perp = binance_btc_usdt_perp.clone();
    binance_btc_usdt_perp.id = Uuid::new_v4();
    instrument_repo
        .insert(&database.pool, binance_btc_usdt_perp.clone())
        .await
        .unwrap();
}

#[rstest]
#[tokio::test]
async fn test_read_instrument(database: PersistanceManager, binance_btc_usdt_perp: Instrument) {
    test_setup();
    let instrument_repo = database.instrument_repo();
    let instrument = instrument_repo
        .read_by_id(
            &database.pool,
            Uuid::from_str("f5dd7db6-89da-4c68-b62e-6f80b763bef6").expect("Invalid UUID"),
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(instrument.id, binance_btc_usdt_perp.id);
}

// #[rstest]
// #[tokio::test]
// async fn test_read_ticks(database: DBManager) {
//     test_setup();
//     let from = datetime!(2024-07-01 00:00).assume_utc();
//     let till = datetime!(2024-07-01 00:10).assume_utc();
//     let ticks = database.read_ticks(&from, &till).await;
//     assert_eq!(ticks.len(), 1);
// }

// #[rstest]
// #[tokio::test]
// async fn test_read_trades(database: DBManager) {
//     test_setup();
//     let from = datetime!(2024-07-01 00:00).assume_utc();
//     let till = datetime!(2024-07-01 00:10).assume_utc();
//     // let ticks = database.read_trades(&from, &till).await;
//     assert_eq!(ticks.len(), 1);
// }
