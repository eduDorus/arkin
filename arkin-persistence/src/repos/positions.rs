use anyhow::Result;
use typed_builder::TypedBuilder;
use sqlx::PgPool;
use uuid::Uuid;

use arkin_core::prelude::*;

#[derive(Debug, Clone, TypedBuilder)]

pub struct PositionsRepo {
    pool: PgPool,
}

impl PositionsRepo {
    pub async fn insert(&self, position: Position) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO positions
            (
                id, 
                instance_id, 
                strategy_id, 
                instrument_id, 
                side, 
                open_price, 
                open_quantity, 
                close_price, 
                close_quantity, 
                realized_pnl, 
                total_commission, 
                status, 
                created_at, 
                updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            position.id,
            position.instance.id,
            position.strategy.id,
            position.instrument.id,
            position.side as PositionSide,
            position.open_price,
            position.open_quantity,
            position.close_price,
            position.close_quantity,
            position.realized_pnl,
            position.total_commission,
            position.status as PositionStatus,
            position.created_at,
            position.updated_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update(&self, position: Position) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE positions
            SET
                open_price = $2, 
                open_quantity = $3, 
                close_price = $4, 
                close_quantity = $5, 
                realized_pnl = $6, 
                total_commission = $7, 
                status = $8, 
                updated_at = $9
            WHERE id = $1
            "#,
            position.id,
            position.open_price,
            position.open_quantity,
            position.close_price,
            position.close_quantity,
            position.realized_pnl,
            position.total_commission,
            position.status as PositionStatus,
            position.updated_at,
        )
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn delete(&self, id: Uuid) -> Result<()> {
        sqlx::query!(
            r#"
            DELETE FROM positions
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
    async fn test_positions_repo() {
        let pool = connect_database();
        let repo = PositionsRepo::builder().pool(pool).build().unwrap();

        let instrument = test_inst_binance_btc_usdt_perp();
        let instance = test_instance();
        let strategy = test_strategy();

        let mut position = Position::builder()
            .id(Uuid::new_v4())
            .instance(instance.clone())
            .strategy(strategy.clone())
            .instrument(instrument)
            .side(PositionSide::Long)
            .last_price(dec!(100))
            .open_price(dec!(100))
            .open_quantity(dec!(1))
            .close_price(dec!(0))
            .close_quantity(dec!(0))
            .realized_pnl(dec!(0))
            .total_commission(dec!(0))
            .status(PositionStatus::Open)
            .created_at(OffsetDateTime::now_utc())
            .updated_at(OffsetDateTime::now_utc())
            .build()
            .unwrap();
        repo.insert(position.clone()).await.unwrap();

        position.close_price = dec!(110);
        position.close_quantity = dec!(1);
        position.realized_pnl = dec!(10);
        position.total_commission = dec!(0.1);
        position.status = PositionStatus::Closed;
        position.updated_at = OffsetDateTime::now_utc();

        repo.update(position.clone()).await.unwrap();
        repo.delete(position.id).await.unwrap();
    }
}
