use anyhow::Result;
use sqlx::PgPool;
use tracing::debug;

use arkin_core::prelude::*;

use crate::BIND_LIMIT;

#[derive(Debug)]
pub struct SignalsRepo {
    pool: PgPool,
}

impl SignalsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, signal: Signal) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO signals
            (
                event_time, 
                instance_id,
                instrument_id, 
                strategy_id, 
                weight
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
            signal.event_time,
            signal.instance.id,
            signal.instrument.id,
            signal.strategy.id,
            signal.weight,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, signals: Vec<Signal>) -> Result<()> {
        // Build batched insert queries
        for batch in signals.chunks(BIND_LIMIT / 5) {
            // Create a query builder
            let mut query_builder = sqlx::QueryBuilder::new(
                r#"
                INSERT INTO signals
                (
                    event_time, 
                    instrument_id, 
                    strategy_id, 
                    weight
                ) 
                "#,
            );

            // Note that `.into_iter()` wasn't needed here since `users` is already an iterator.
            query_builder.push_values(batch, |mut b, signal| {
                // If you wanted to bind these by-reference instead of by-value,
                // you'd need an iterator that yields references that live as long as `query_builder`,
                // e.g. collect it to a `Vec` first.
                b.push_bind(signal.event_time)
                    .push(signal.instance.id)
                    .push_bind(signal.instrument.id)
                    .push_bind(signal.strategy.id)
                    .push_bind(signal.weight);
            });

            let query = query_builder.build();

            query.execute(&self.pool).await?;
        }
        debug!("Saved {} venue signals", signals.len());
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use crate::test_utils::connect_database;

    use super::*;
    use rust_decimal_macros::dec;
    use test_log::test;
    use time::OffsetDateTime;

    #[test(tokio::test)]
    async fn test_signals_repo() {
        let pool = connect_database();
        let repo = SignalsRepo::new(pool);

        let instrument = test_inst_binance_btc_usdt_perp();
        let instance = test_instance();
        let strategy = test_strategy();

        let signal = Signal {
            event_time: OffsetDateTime::now_utc(),
            instance,
            instrument,
            strategy,
            weight: dec!(0.5),
        };
        repo.insert(signal).await.unwrap();
    }
}
