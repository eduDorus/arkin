use std::sync::Arc;

use derive_builder::Builder;
use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use tracing::debug;

use arkin_core::prelude::*;
use uuid::Uuid;

use crate::{PersistenceError, BIND_LIMIT};

const FIELD_COUNT: usize = 5;

#[derive(Debug, FromRow)]
pub struct SignalDTO {
    pub event_time: OffsetDateTime,
    pub instrument_id: Uuid,
    pub strategy_id: Uuid,
    pub weight: Decimal,
}

impl From<Arc<Signal>> for SignalDTO {
    fn from(signal: Arc<Signal>) -> Self {
        Self {
            event_time: signal.event_time,
            instrument_id: signal.instrument.id,
            strategy_id: signal.strategy.id,
            weight: signal.weight,
        }
    }
}

#[derive(Debug, Clone, Builder)]
#[builder(setter(into))]
pub struct SignalRepo {
    pool: PgPool,
}

impl SignalRepo {
    pub async fn insert(&self, signal: SignalDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO signals
            (
                event_time, 
                strategy_id, 
                instrument_id, 
                weight
            ) VALUES ($1, $2, $3, $4)
            "#,
            signal.event_time,
            signal.strategy_id,
            signal.instrument_id,
            signal.weight,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, signals: Vec<SignalDTO>) -> Result<(), PersistenceError> {
        // Build batched insert queries
        for batch in signals.chunks(BIND_LIMIT / FIELD_COUNT) {
            // Create a query builder
            let mut query_builder = sqlx::QueryBuilder::new(
                r#"
                INSERT INTO signals
                (
                    event_time, 
                    strategy_id, 
                    instrument_id, 
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
                    .push_bind(signal.strategy_id)
                    .push_bind(signal.instrument_id)
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
        let repo = SignalRepoBuilder::default().pool(pool).build().unwrap();

        let signal = Arc::new(
            SignalBuilder::default()
                .event_time(OffsetDateTime::now_utc())
                .instrument(test_inst_binance_btc_usdt_perp())
                .strategy(test_strategy())
                .weight(dec!(0.5))
                .build()
                .unwrap(),
        );
        repo.insert(signal.into()).await.unwrap();
    }
}
