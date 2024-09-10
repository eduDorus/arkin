use anyhow::Result;
use arkin_core::prelude::*;
use futures_util::StreamExt;
use rust_decimal::Decimal;
use time::OffsetDateTime;
use tracing::error;

use crate::db::DBManager;

#[derive(sqlx::FromRow)]
struct TradeRow {
    received_time: OffsetDateTime,
    event_time: OffsetDateTime,
    instrument_type: String,
    venue: String,
    base: String,
    quote: String,
    maturity: Option<OffsetDateTime>,
    strike: Option<Decimal>,
    option_type: Option<String>,
    trade_id: i64,
    price: Decimal,
    quantity: Decimal,
}

impl From<Trade> for TradeRow {
    fn from(trade: Trade) -> Self {
        Self {
            received_time: trade.received_time,
            event_time: trade.event_time,
            instrument_type: trade.instrument.instrument_type().to_string(),
            venue: trade.instrument.venue().to_string(),
            base: trade.instrument.base().to_string(),
            quote: trade.instrument.quote().to_string(),
            maturity: trade.instrument.maturity().map(|m| m.to_owned()),
            strike: trade.instrument.strike().map(|s| s.to_owned()),
            option_type: trade.instrument.option_type().map(|ot| ot.to_string()),
            trade_id: trade.trade_id as i64,
            price: trade.price,
            quantity: trade.quantity,
        }
    }
}

impl From<TradeRow> for Trade {
    fn from(db_trade: TradeRow) -> Self {
        let instrument = match db_trade.instrument_type.as_str() {
            "perp" => Instrument::perpetual(
                db_trade.venue.parse().expect("Failed to parse venue"),
                db_trade.base.as_str().into(),
                db_trade.quote.as_str().into(),
            ),
            "future" => Instrument::future(
                db_trade.venue.parse().expect("Failed to parse venue"),
                db_trade.base.as_str().into(),
                db_trade.quote.as_str().into(),
                db_trade.maturity.expect("Failed to parse maturity"),
            ),
            "option" => Instrument::option(
                db_trade.venue.parse().expect("Failed to parse venue"),
                db_trade.base.as_str().into(),
                db_trade.quote.as_str().into(),
                db_trade.strike.expect("Failed to parse strike"),
                db_trade.maturity.expect("Failed to parse maturity"),
                db_trade
                    .option_type
                    .as_ref()
                    .map(|ot| ot.parse().expect("Failed to parse option type"))
                    .unwrap(),
            ),
            _ => panic!("Invalid instrument type"),
        };

        Trade::new(
            db_trade.received_time,
            db_trade.event_time,
            instrument,
            db_trade.trade_id as u64,
            db_trade.price.into(),
            db_trade.quantity.into(),
        )
    }
}

impl DBManager {
    pub async fn insert_trade(&self, trade: Trade) -> Result<()> {
        sqlx::query!(
            r#"
            WITH existing_instrument AS (
                SELECT instrument_id
                FROM instruments
                WHERE instrument_type = $3
                AND venue = $4
                AND base = $5
                AND quote = $6
                AND maturity IS NOT DISTINCT FROM $7
                AND strike IS NOT DISTINCT FROM $8
                AND option_type IS NOT DISTINCT FROM $9
            ), insert_instrument AS (
                INSERT INTO instruments (instrument_type, venue, base, quote, maturity, strike, option_type)
                SELECT $3, $4, $5, $6, $7, $8, $9
                WHERE NOT EXISTS (SELECT 1 FROM existing_instrument)
                RETURNING instrument_id
            )
            INSERT INTO trades (
                received_time, event_time, instrument_id, trade_id, price, quantity
            )
            SELECT
                $1, $2, COALESCE(ei.instrument_id, ii.instrument_id), $10, $11, $12
            FROM
                existing_instrument ei
            FULL OUTER JOIN
                insert_instrument ii ON true
            LIMIT 1
            "#,
            trade.received_time,
            trade.event_time,
            trade.instrument.instrument_type().to_string(),
            trade.instrument.venue().to_string(),
            trade.instrument.base().to_string(),
            trade.instrument.quote().to_string(),
            trade.instrument.maturity().map(|m| m.to_owned()),
            trade.instrument.strike().map(|s| s.to_owned()),
            trade.instrument.option_type().map(|ot| ot.to_string()),
            trade.trade_id as i64,
            trade.price,
            trade.quantity,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn read_trades(&self, from: &OffsetDateTime, to: &OffsetDateTime) -> Vec<Trade> {
        let stream = sqlx::query_as!(
            TradeRow,
            r#"
            SELECT
                t.received_time,
                t.event_time,
                i.instrument_type,
                i.venue,
                i.base,
                i.quote,
                i.maturity,
                i.strike,
                i.option_type,
                t.trade_id,
                t.price,
                t.quantity
            FROM trades as t
            JOIN instruments as i ON t.instrument_id = i.instrument_id
            WHERE i.instrument_id in (1, 2) AND t.event_time >= $1 AND t.event_time < $2
            "#,
            from,
            to
        )
        .fetch(&self.pool);

        stream
            .filter_map(|res| async {
                match res {
                    Ok(v) => Some(v.into()),
                    Err(e) => {
                        error!("Error reading tick: {:?}", e);
                        None
                    }
                }
            })
            .collect()
            .await
    }
}

// #[cfg(test)]
// mod tests {
//     use rust_decimal::prelude::*;
//     use time::OffsetDateTime;
//     use tracing::info;

//     use super::*;
//     use crate::{
//         config,
//         ingestors::IngestorID,
//         logging,
//         models::{Instrument, Venue},
//     };

//     #[tokio::test]
//     #[ignore]
//     async fn test_insert_trade() {
//         let config = config::load();
//         let manager = DBManager::from_config(&config.db).await;

//         let trade = Trade {
//             received_time: OffsetDateTime::now_utc(),
//             event_time: OffsetDateTime::now_utc(),
//             instrument: Instrument::perpetual(Venue::Binance, "BTC".into(), "USDT".into()),
//             trade_id: 1,
//             price: Decimal::new(10000, 2).into(),
//             quantity: Decimal::new(105, 1).into(),
//             source: IngestorID::Test,
//         };

//         manager.insert_trade(trade).await.unwrap();

//         // Check that the trade was inserted
//         let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM trades")
//             .fetch_one(&manager.pool)
//             .await
//             .expect("SQLX failed to fetch row");
//         assert_eq!(row.0, 1)
//     }

//     #[tokio::test]
//     #[ignore]
//     async fn test_read_trades() {
//         logging::init_test_tracing();

//         let config = config::load();
//         let manager = DBManager::from_config(&config.db).await;

//         let till = OffsetDateTime::now_utc();
//         let from = till - time::Duration::days(1);

//         let trades = manager.read_trades(&from, &till).await;
//         assert_eq!(trades.len(), 2);
//         for trade in trades {
//             info!("{}", trade);
//         }
//     }
// }
