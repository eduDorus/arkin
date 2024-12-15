use std::sync::Arc;

use rust_decimal::Decimal;
use sqlx::{prelude::*, PgPool};
use time::OffsetDateTime;
use tracing::debug;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::Insight;

use crate::{PersistenceError, BIND_LIMIT};

const FIELD_COUNT: usize = 6;

#[derive(Debug, Clone, FromRow)]
pub struct InsightDTO {
    pub id: Uuid,
    pub event_time: OffsetDateTime,
    pub pipeline_id: Uuid,
    pub instrument_id: Option<Uuid>,
    pub feature_id: String,
    pub value: Decimal,
}

impl From<Arc<Insight>> for InsightDTO {
    fn from(insight: Arc<Insight>) -> Self {
        Self {
            id: insight.id,
            event_time: insight.event_time,
            pipeline_id: insight.pipeline.id,
            instrument_id: insight.instrument.as_ref().map(|i| i.id),
            feature_id: insight.feature_id.to_string(),
            value: insight.value,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct InsightsRepo {
    pool: PgPool,
}

impl InsightsRepo {
    pub async fn insert(&self, insight: InsightDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO insights 
            (
                id,
                event_time, 
                pipeline_id,
                instrument_id, 
                feature_id, 
                value
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (event_time, pipeline_id, instrument_id, feature_id)
            DO UPDATE SET value = EXCLUDED.value
            "#,
            insight.id,
            insight.event_time,
            insight.pipeline_id,
            insight.instrument_id,
            insight.feature_id,
            insight.value,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, insights: &[InsightDTO]) -> Result<(), PersistenceError> {
        // Build batched insert queries
        for batch in insights.chunks(BIND_LIMIT / FIELD_COUNT) {
            // Create a query builder
            let mut query_builder = sqlx::QueryBuilder::new(
                r#"
                INSERT INTO insights 
                (
                    id,
                    event_time, 
                    pipeline_id,
                    instrument_id, 
                    feature_id, 
                    value
                ) 
                "#,
            );

            // Note that `.into_iter()` wasn't needed here since `users` is already an iterator.
            query_builder.push_values(batch, |mut b, insight| {
                // If you wanted to bind these by-reference instead of by-value,
                // you'd need an iterator that yields references that live as long as `query_builder`,
                // e.g. collect it to a `Vec` first.
                b.push_bind(insight.id)
                    .push_bind(insight.event_time)
                    .push_bind(insight.pipeline_id)
                    .push_bind(insight.instrument_id)
                    .push_bind(insight.feature_id.clone())
                    .push_bind(insight.value);
            });

            query_builder.push(
                "ON CONFLICT (event_time, pipeline_id, instrument_id, feature_id) DO UPDATE SET value = EXCLUDED.value",
            );
            let query = query_builder.build();

            query.execute(&self.pool).await?;
        }
        debug!("Saved {} insights", insights.len());
        Ok(())
    }
}
