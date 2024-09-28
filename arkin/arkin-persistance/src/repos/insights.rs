use anyhow::Result;
use futures_util::{stream, StreamExt};
use rust_decimal::Decimal;
use sqlx::{prelude::*, PgPool};
use time::OffsetDateTime;
use tracing::error;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::{BIND_LIMIT, MAX_CONCURRENT_QUERIES};

#[derive(Debug, FromRow)]
pub struct DBInsight {
    pub instrument_id: Uuid,
    pub event_time: OffsetDateTime,
    pub feature_id: String,
    pub value: Decimal,
}

impl From<Insight> for DBInsight {
    fn from(insight: Insight) -> Self {
        Self {
            instrument_id: insight.instrument.id,
            event_time: insight.event_time,
            feature_id: insight.id,
            value: insight.value,
        }
    }
}

pub struct InsightsRepo {
    pool: PgPool,
}

impl InsightsRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, insight: Insight) -> Result<()> {
        let insight = DBInsight::from(insight);
        sqlx::query!(
            r#"
            INSERT INTO insights (instrument_id, event_time, feature_id, value)
            VALUES ($1, $2, $3, $4)
            "#,
            insight.instrument_id,
            insight.event_time,
            insight.feature_id,
            insight.value,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, insights: Vec<Insight>) -> Result<()> {
        let insights = insights.into_iter().map(DBInsight::from).collect::<Vec<_>>();

        let queries = insights
            .chunks(BIND_LIMIT / 4)
            .map(|batch| {
                // Create a query builder
                let mut query_builder =
                    sqlx::QueryBuilder::new("INSERT INTO insights (instrument_id, event_time, feature_id, value) ");

                // Note that `.into_iter()` wasn't needed here since `users` is already an iterator.
                query_builder.push_values(batch, |mut b, insight| {
                    // If you wanted to bind these by-reference instead of by-value,
                    // you'd need an iterator that yields references that live as long as `query_builder`,
                    // e.g. collect it to a `Vec` first.
                    b.push_bind(insight.instrument_id)
                        .push_bind(insight.event_time)
                        .push_bind(insight.feature_id.clone())
                        .push_bind(insight.value);
                });

                query_builder
            })
            .collect::<Vec<_>>();

        let query_stream = stream::iter(queries.into_iter().map(|mut query| {
            let db_pool = self.pool.clone();
            async move { query.build().execute(&db_pool).await }
        }));

        let results = query_stream.buffered(MAX_CONCURRENT_QUERIES).collect::<Vec<_>>().await;

        for result in results {
            match result {
                Ok(_) => { /* Success */ }
                Err(e) => {
                    error!("Error executing query: {}", e);
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }
}
