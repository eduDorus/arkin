use std::sync::Arc;

use rust_decimal::Decimal;
use sqlx::{FromRow, PgPool};
use time::OffsetDateTime;
use typed_builder::TypedBuilder;
use uuid::Uuid;

use arkin_core::prelude::*;

use crate::{PersistenceError, BIND_LIMIT};

const FIELD_COUNT: usize = 9;

#[derive(Debug, FromRow)]
pub struct TransactionDTO {
    pub event_time: OffsetDateTime,
    pub transaction_group_id: Uuid,
    pub portfolio_id: Uuid,
    pub asset_id: Option<Uuid>,
    pub instrument_id: Option<Uuid>,
    pub transaction_type: TransactionType,
    pub price: Option<Decimal>,
    pub quantity: Decimal,
    pub total_value: Decimal,
}

impl From<Arc<Transaction>> for TransactionDTO {
    fn from(transaction: Arc<Transaction>) -> Self {
        Self {
            event_time: transaction.event_time,
            transaction_group_id: transaction.transaction_group_id,
            portfolio_id: transaction.portfolio.id,
            asset_id: transaction.asset.as_ref().map(|a| a.id),
            instrument_id: transaction.instrument.as_ref().map(|i| i.id),
            transaction_type: transaction.transaction_type.clone(),
            price: transaction.price,
            quantity: transaction.quantity,
            total_value: transaction.total_value,
        }
    }
}

#[derive(Debug, Clone, TypedBuilder)]

pub struct TransactionRepo {
    pool: PgPool,
}

impl TransactionRepo {
    pub async fn insert(&self, transaction: TransactionDTO) -> Result<(), PersistenceError> {
        sqlx::query!(
            r#"
            INSERT INTO transactions
            (
                event_time, 
                transaction_group_id, 
                portfolio_id, 
                asset_id, 
                instrument_id, 
                transaction_type, 
                price, 
                quantity, 
                total_value
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            transaction.event_time,
            transaction.transaction_group_id,
            transaction.portfolio_id,
            transaction.asset_id,
            transaction.instrument_id,
            transaction.transaction_type as TransactionType,
            transaction.price,
            transaction.quantity,
            transaction.total_value,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_batch(&self, transactions: Vec<TransactionDTO>) -> Result<(), PersistenceError> {
        // Build batched insert queries
        for batch in transactions.chunks(BIND_LIMIT / FIELD_COUNT) {
            // Create a query builder
            let mut query_builder = sqlx::QueryBuilder::new(
                r#"
                INSERT INTO transactions
                (
                    event_time, 
                    transaction_group_id, 
                    portfolio_id, 
                    asset_id, 
                    instrument_id, 
                    transaction_type, 
                    price, 
                    quantity, 
                    total_value
                ) 
                "#,
            );

            // Push the values into the query builder
            query_builder.push_values(batch, |mut b, tick| {
                b.push_bind(tick.event_time)
                    .push_bind(tick.transaction_group_id)
                    .push_bind(tick.portfolio_id)
                    .push_bind(tick.asset_id)
                    .push_bind(tick.instrument_id)
                    .push_bind(tick.transaction_type)
                    .push_bind(tick.price)
                    .push_bind(tick.quantity)
                    .push_bind(tick.total_value);
            });

            // Use ON CONFLICT for the composite primary key
            // query_builder.push("ON CONFLICT (instrument_id, tick_id, event_time) DO NOTHING");
            let query = query_builder.build();

            query.execute(&self.pool).await?;
        }
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
    use uuid::Uuid;

    #[test(tokio::test)]
    async fn test_transaction_repo() {
        let pool = connect_database();
        let repo = TransactionRepo::builder().pool(pool).build();

        let transaction = Transaction::builder()
            .event_time(OffsetDateTime::now_utc())
            .transaction_group_id(Uuid::new_v4())
            .portfolio(test_portfolio())
            .asset(Some(usdt_asset()))
            .instrument(None)
            .transaction_type(TransactionType::Collateral)
            .price(Some(dec!(1)))
            .quantity(dec!(100))
            .total_value(dec!(100))
            .build();
        let transaction = Arc::new(transaction);
        repo.insert(transaction.clone().into()).await.unwrap();
    }
}
