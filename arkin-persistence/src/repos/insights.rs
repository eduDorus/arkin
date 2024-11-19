use anyhow::Result;
use rust_decimal::Decimal;
use sqlx::{prelude::*, PgPool};
use time::OffsetDateTime;
use tracing::debug;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::BIND_LIMIT;

#[derive(Debug, FromRow)]
pub struct DBInsight {
    pub instrument_id: Option<Uuid>,
    pub event_time: OffsetDateTime,
    pub feature_id: String,
    pub value: Decimal,
}

impl From<Insight> for DBInsight {
    fn from(insight: Insight) -> Self {
        Self {
            instrument_id: match insight.instrument {
                Some(i) => Some(i.id),
                None => None,
            },
            event_time: insight.event_time,
            feature_id: insight.feature_id.as_ref().clone(),
            value: insight.value,
        }
    }
}

#[derive(Debug)]
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
            INSERT INTO insights (event_time, instrument_id, feature_id, value)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (event_time, instrument_id, feature_id)
            DO UPDATE SET value = EXCLUDED.value
            "#,
            insight.event_time,
            insight.instrument_id,
            insight.feature_id,
            insight.value,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, insights: Vec<Insight>) -> Result<()> {
        let db_insights = insights.into_iter().map(DBInsight::from).collect::<Vec<_>>();

        // Build batched insert queries
        for batch in db_insights.chunks(BIND_LIMIT / 4) {
            // Create a query builder
            let mut query_builder =
                sqlx::QueryBuilder::new("INSERT INTO insights (event_time, instrument_id, feature_id, value) ");

            // Note that `.into_iter()` wasn't needed here since `users` is already an iterator.
            query_builder.push_values(batch, |mut b, insight| {
                // If you wanted to bind these by-reference instead of by-value,
                // you'd need an iterator that yields references that live as long as `query_builder`,
                // e.g. collect it to a `Vec` first.
                b.push_bind(insight.event_time)
                    .push_bind(insight.instrument_id)
                    .push_bind(insight.feature_id.clone())
                    .push_bind(insight.value);
            });

            query_builder
                .push("ON CONFLICT (event_time, instrument_id, feature_id) DO UPDATE SET value = EXCLUDED.value");
            let query = query_builder.build();

            query.execute(&self.pool).await?;
        }
        debug!("Saved {} inserts", db_insights.len());
        Ok(())
    }

    // I DON'T THINK WE WILL EVER READ INSIGHTS INTO OUR SYSTEM
    // pub async fn read_range_by_instrument_id_and_feature_id(
    //     &self,
    //     instrument_id: Uuid,
    //     feature_id: &str,
    //     start: OffsetDateTime,
    //     end: OffsetDateTime,
    // ) -> Result<Vec<DBInsight>> {
    //     let insights = sqlx::query_as!(
    //         DBInsight,
    //         r#"
    //         SELECT * FROM insights
    //         WHERE instrument_id = $1 AND feature_id = $2 AND event_time >= $3 AND event_time < $4
    //         ORDER BY event_time ASC
    //         "#,
    //         instrument_id,
    //         feature_id,
    //         start,
    //         end,
    //     )
    //     .fetch_all(&self.pool)
    //     .await?;

    //     Ok(insights)
    // }
}