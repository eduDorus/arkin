use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use arkin_core::prelude::*;

#[derive(Debug)]
pub struct ExecutionOrdersRepo {
    pool: PgPool,
}

impl ExecutionOrdersRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert(&self, order: ExecutionOrder) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO execution_orders
            (
                id, 
                instance_id, 
                strategy_id, 
                instrument_id, 
                order_type, 
                side, 
                price, 
                quantity, 
                fill_price, 
                filled_quantity, 
                total_commission, 
                status, 
                created_at, 
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            order.id,
            order.instance.id,
            order.strategy.id,
            order.instrument.id,
            order.order_type as ExecutionOrderType,
            order.side as MarketSide,
            order.price,
            order.quantity,
            order.fill_price,
            order.filled_quantity,
            order.total_commission,
            order.status as ExecutionOrderStatus,
            order.created_at,
            order.updated_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update(&self, order: ExecutionOrder) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE execution_orders
            SET
                fill_price = $2,
                filled_quantity = $3,
                total_commission = $4,
                status = $5,
                updated_at = $6
            WHERE id = $1
            "#,
            order.id,
            order.fill_price,
            order.filled_quantity,
            order.total_commission,
            order.status as ExecutionOrderStatus,
            order.updated_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: ExecutionOrderId) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM execution_orders
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;
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
    async fn test_execution_order_repo() {
        let pool = connect_database();
        let repo = ExecutionOrdersRepo::new(pool);

        let instance = test_instance();
        let strategy = test_strategy();
        let instrument = test_inst_binance_btc_usdt_perp();

        let mut order = ExecutionOrderBuilder::default()
            .id(Uuid::new_v4())
            .instance(instance.clone())
            .strategy(strategy.clone())
            .instrument(instrument.clone())
            .order_type(ExecutionOrderType::Maker)
            .side(MarketSide::Buy)
            .price(dec!(0))
            .quantity(dec!(1))
            .build()
            .unwrap();
        repo.insert(order.clone()).await.unwrap();

        order.fill_price = dec!(110);
        order.filled_quantity = dec!(1);
        order.total_commission = dec!(0.2);
        order.status = ExecutionOrderStatus::Filled;
        order.updated_at = OffsetDateTime::now_utc();

        repo.update(order.clone()).await.unwrap();
        repo.delete(order.id).await.unwrap();
    }
}
